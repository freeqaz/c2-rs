//! `c2rs` — CLI over the differential harness. std only (no clap): args are
//! parsed by hand. Every subcommand degrades to "SKIP: toolchain absent" when
//! `Toolchain::locate()` is `None` — it never panics on a missing toolchain.
//!
//! Subcommands:
//!   capture <cpp>       capture IL, print the 5 file sizes and the profile used
//!                       (--flags-file / --cwd for a real-project profile)
//!   compile <cpp>       reference obj, print size + timestamp
//!   selftest [<cpp>...] oracle self-test over the given TUs (or all fixtures)
//!   replay <cpp>        P0.1: capture + standalone-c2 replay, print byte-match
//!   replay-c1 <cpp>     P-F0.1: capture + standalone-c1 (front-end) replay, per-file byte verdict
//!   diff <cpp>          full differential (ReferenceReplay=ByteExact, Port=Match|NotImplemented)
//!   bench               selftest across all fixtures/cpp/*.cpp, summary counts
//!   perf                IL-bundle->obj latency: native port vs standalone c2
//!   perf-scale          IL-bundle->obj throughput vs concurrency (port vs c2)
//!   corpus <sub>        P1.2 corpus generator (gen / sample / stats)

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::corpus::{self, CorpusConfig};
use c2_harness::prefilter;
use c2_harness::provenance::Provenance;
use c2_harness::retrieval;
use c2_harness::{
    all_fixtures, c1_replay_check, differential, oracle_selftest, C1ReplayReport, DiffReport,
    PortStatus, SelfTestOutcome, SelfTestReport,
};
use c2_il::IL_SUFFIXES;
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("c2rs-cli-{tag}-{}-{}-{}", std::process::id(), nanos, n));
    let _ = std::fs::create_dir_all(&d);
    d
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let rest = &args[args.len().min(1)..];

    match cmd {
        "capture" => cmd_capture(rest),
        "compile" => cmd_compile(rest),
        "selftest" => cmd_selftest(rest),
        "replay" => cmd_replay(rest),
        "replay-c1" => cmd_replay_c1(rest),
        "diff" => cmd_diff(rest),
        "census" => cmd_census(rest),
        "bench" => cmd_bench(rest),
        "perf" => cmd_perf(rest),
        "perf-scale" => cmd_perf_scale(rest),
        "corpus" => cmd_corpus(rest),
        "gap" => cmd_gap(rest),
        "listing" => cmd_listing(rest),
        "listing-scan" => cmd_listing_scan(rest),
        "prefilter" => cmd_prefilter(rest),
        "retrieve" => cmd_retrieve(rest),
        "search" => cmd_search(rest),
        "help" | "-h" | "--help" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}\n");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    eprintln!(
        "c2rs — differential harness for the c2.dll native port\n\
         \n\
         USAGE:\n\
         \x20 c2rs capture <cpp> [--keep-il DIR] [--flags-file F] [--cwd DIR]\n\
         \x20                           capture IL, print the 5 file sizes and the profile used.\n\
         \x20                           WITHOUT --flags-file the profile is /Ox /GS- /c, which is\n\
         \x20                           NOT the workload's (/O1 /Oi /EHsc /GR ...) and does not\n\
         \x20                           imply /GF — pass the workload's flags.txt to compare a\n\
         \x20                           captured .gl against a workload obj.\n\
         \x20 c2rs compile <cpp> [--keep-obj PATH] [--flags-file F] [--cwd DIR]\n\
         \x20                           reference obj, print size + timestamp and the profile used.\n\
         \x20                           WITHOUT --flags-file the profile is /Ox /GS- /c, the same\n\
         \x20                           default `capture` has and NOT the workload's; --cwd is\n\
         \x20                           meaningful only together with --flags-file.\n\
         \x20 c2rs selftest [<cpp>...]  oracle self-test (determinism + capture stability)\n\
         \x20 c2rs replay <cpp>         P0.1: capture + standalone-c2 replay, byte-match verdict\n\
         \x20 c2rs replay-c1 <cpp>      P-F0.1: capture + standalone-c1 (front-end) replay, per-file byte verdict\n\
         \x20 c2rs diff <cpp>           full differential (ReferenceReplay=ByteExact, Port=Match|NotImplemented)\n\
         \x20 c2rs census <cpp>         P2b: per-function in-class / blocking-feature verdict\n\
         \x20 c2rs bench                selftest across all fixtures/cpp/*.cpp\n\
         \x20 c2rs perf [opts]          IL-bundle->obj latency: native port vs standalone c2\n\
         \x20 c2rs perf-scale [opts]    IL-bundle->obj throughput vs concurrency (port vs c2)\n\
         \x20 c2rs corpus gen [opts]    P1.2: generate a (source,IL,obj) triple corpus\n\
         \x20 c2rs corpus sample [dir]  write the portable synthetic sample corpus\n\
         \x20 c2rs corpus stats <dir>   summarize a corpus manifest\n\
         \x20 c2rs gap [opts]           real-workload gap scan: classify every TU, rank the blockers\n\
         \x20 c2rs listing <cpp> [opts] board #132: capture c2's own .cod assembly listing beside the obj\n\
         \x20 c2rs listing-scan [opts]  boards #134/#136: /QXSTALLS demand + the .cod census reconcile\n\
         \x20 c2rs prefilter [opts]     reject-only pre-filter seam: one JSON verdict for one candidate TU\n\
         \x20 c2rs retrieve index <dir> P1.3: obj-retrieval structure of a corpus\n\
         \x20 c2rs retrieve eval <dir>  P1.3: obj->IL retrieval baseline, recall@k\n\
         \x20 c2rs search solve <cpp>   T-A: solve one d=1 instance from a fixture, byte-exact\n\
         \x20 c2rs search eval [opts]   T-A: IL-space solve-rate over fixtures\n\
         \x20 c2rs search from-retrieval <corpus-dir>  T-A: from-unrelated-seed (P1.3-seeded) solve-rate\n\
         \n\
         perf options: --port-iters N --ref-iters N --fixtures a.cpp,b.cpp\n\
         census: c2rs census <cpp> — per-function in-class/blocked verdict (P2b)\n\
         perf-scale options: --fixture X.cpp --conc 1,2,4,8 --port-secs F --ref-secs F --csv PATH\n\
         corpus gen options: --seed N --count N --out DIR --timeout SECS\n\
         gap options: --list FILE --flags-file FILE [--cwd DIR] [--limit N] [--jobs N]\n\
         \x20            [--replay-every N] [--jsonl PATH] (see scripts/gen_dc3_workload.sh)\n\
         \x20            [--cache DIR | --no-cache] [--validate-cache N]\n\
         \x20            captures are cached content-addressed (source bytes + flags +\n\
         \x20            toolchain + workload git identity, never mtimes) under\n\
         \x20            <main-repo>/work/capture-cache (shared by every worktree)\n\
         \x20            or C2RS_GAP_CACHE; --validate-cache N\n\
         \x20            re-captures every Nth hit and byte-compares it.\n\
         listing options: [--qxstalls] [--out PATH] [--flag F ...]  (default flags /O1 /Oi /EHsc /GS- /c)\n\
         listing-scan options: --list FILE --flags-file FILE [--cwd DIR] [--limit N] [--jobs N]\n\
         \x20                    [--qxstalls] [--jsonl PATH] [--work DIR]\n\
         prefilter options: --source ARG (--flag F ... | --flags-file FILE) [--cwd DIR]\n\
         \x20                 [--emit-obj PATH] [--compare-obj PATH] [--obj-name Z:\\\\...] [--work DIR]\n\
         retrieve eval options: --split held-out|loo --query-div N --k 1,5,10\n\
         search options: --moves full|length --steps N --compiles N --beam K --timeout SECS\n\
         \x20              --d 1|2|3 (eval only — solve hardcodes d=1 and never read it)\n\
         \x20              from-retrieval: --sample N --multi N --select-seed N\n\
         \x20              from-lifter: --gens FILE --k K --limit N\n\
         \n\
         Toolchain: compilers/ via scripts/fetch_compilers.sh (or C2RS_COMPILERS /\n\
         C2RS_CL_EXE / C2RS_C2_DLL / C2RS_C1XX_DLL), wibo via C2RS_WIBO, sibling\n\
         ../wibo build, or PATH. Absent toolchain -> clean SKIP."
    );
}

/// **The argument seam.** One parser for every subcommand — and the only place
/// in this binary that can produce a [`Toolchain`].
///
/// # Why this is a module and not a helper function
///
/// Boards #194 and #195 were two instances of one class, and a sweep across all
/// **26** dispatch arms found it at fourteen more. The class has two halves and
/// they are separate defects with one cure:
///
/// 1. **Scan instead of parse.** `iter().position(|a| a == "--x")` — and the
///    `opt()` helper, which was the same scan under a nicer name — only ever
///    *looks for* keys it is asked about, so **every other argument is invisible
///    by construction**. `c2rs compile <cpp> --flag /GR-` dropped `--flag` and
///    ran two identical command lines, and the identical objs were read as a
///    finding about RTTI. Two different commands producing one output is
///    indistinguishable, at the terminal, from a real negative result.
/// 2. **Locate before validate.** Eight handlers called `located()` *before*
///    looking at their arguments. `located()` returns `None` and the handler
///    exits **0** with `SKIP: toolchain absent` — so on a machine with no
///    compilers, which is exactly where the portable test lane runs, a
///    completely bogus command line **passed**. A test cannot catch a usage
///    error that the binary never reports.
///
/// The cure for (2) is structural rather than conventional: [`Args::toolchain`]
/// is the only producer of a `Toolchain` in this file, and an [`Args`] can only
/// be obtained from [`Args::parse`]. **"Parse and validate, then locate" is
/// therefore the only order this binary can express** — a handler that wants a
/// toolchain must already hold a fully-validated argument set. The free
/// `located()` this replaces was callable from anywhere, which is precisely why
/// eight handlers called it first.
///
/// `tests/cli_flags.rs::locate_is_reachable_only_through_the_arg_seam` is the
/// backstop: `Toolchain::locate` may appear in this file **only** inside this
/// module. Convention plus a check, because a convention nobody checks is how
/// this class got to fourteen sites.
mod argv {
    use super::Toolchain;
    use std::path::PathBuf;
    use std::process::ExitCode;
    use std::str::FromStr;

    /// How many values an option takes, and whether it may repeat.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Arity {
        /// `--qxstalls` — presence only.
        Flag,
        /// `--jobs 8` — one value, and **repeating it is an error**. A silent
        /// first-wins (what `position()` did) means `--k 1 --k 20` runs at 1
        /// while the terminal shows 20.
        Value,
        /// `--flag /GR-` given any number of times; every occurrence is kept.
        Repeated,
    }

    /// A subcommand's grammar. `const`, so the grammar and the usage text are
    /// two renderings of one list rather than two lists that drift.
    pub struct Spec {
        pub cmd: &'static str,
        pub opts: &'static [(&'static str, Arity)],
        /// `(dependent, required)` — `dependent` is *meaningless* without
        /// `required`, so giving it alone is a usage error rather than a
        /// silently dropped argument. `("--cwd", "--flags-file")` is the live
        /// instance: `--cwd` is consumed only on the profile path, so accepting
        /// it alone compiled at a different directory than the one named.
        pub requires: &'static [(&'static str, &'static str)],
        /// Maximum bare positionals. `usize::MAX` for "any number" (`selftest`).
        pub max_positionals: usize,
    }

    impl Spec {
        pub const fn new(cmd: &'static str, opts: &'static [(&'static str, Arity)]) -> Spec {
            Spec { cmd, opts, requires: &[], max_positionals: 1 }
        }
        pub const fn requires(mut self, r: &'static [(&'static str, &'static str)]) -> Spec {
            self.requires = r;
            self
        }
        pub const fn positionals(mut self, n: usize) -> Spec {
            self.max_positionals = n;
            self
        }
    }

    /// A **validated** argument set. The only way to build one is [`Args::parse`],
    /// and the only way to get a [`Toolchain`] is [`Args::toolchain`].
    pub struct Args {
        cmd: &'static str,
        positionals: Vec<String>,
        values: Vec<(&'static str, String)>,
        flags: Vec<&'static str>,
    }

    impl Args {
        /// Parse `rest` against `spec`. **Refuses** an unknown option, a missing
        /// value, a repeated single-valued option, a surplus positional, and a
        /// dependent option whose requirement is absent — each with a message
        /// that **names the argument**, because a refusal that does not say
        /// which argument was rejected leaves the user guessing among several.
        ///
        /// Returns `Err(ExitCode::from(2))` so a handler propagates with `?`-ish
        /// brevity and cannot forget to.
        pub fn parse(spec: &'static Spec, rest: &[String]) -> Result<Args, ExitCode> {
            let cmd = spec.cmd;
            let mut positionals: Vec<String> = Vec::new();
            let mut values: Vec<(&'static str, String)> = Vec::new();
            let mut flags: Vec<&'static str> = Vec::new();

            let mut i = 0usize;
            while i < rest.len() {
                let a = rest[i].as_str();
                if let Some(&(name, arity)) = spec.opts.iter().find(|(n, _)| *n == a) {
                    match arity {
                        Arity::Flag => {
                            if flags.contains(&name) {
                                eprintln!("{cmd}: {name} given more than once");
                                return Err(ExitCode::from(2));
                            }
                            flags.push(name);
                            i += 1;
                        }
                        Arity::Value | Arity::Repeated => {
                            let Some(v) = rest.get(i + 1) else {
                                eprintln!("{cmd}: {name} needs a value");
                                return Err(ExitCode::from(2));
                            };
                            // A value that is itself a known option is almost
                            // always a forgotten argument (`--seed --count 5`
                            // silently made the seed the string "--count"), and
                            // there is no legitimate use of one here.
                            if spec.opts.iter().any(|(n, _)| n == v) {
                                eprintln!(
                                    "{cmd}: {name} needs a value, but the next argument is the \
                                     option {v}"
                                );
                                return Err(ExitCode::from(2));
                            }
                            if arity == Arity::Value && values.iter().any(|(n, _)| *n == name) {
                                eprintln!(
                                    "{cmd}: {name} given more than once; it takes a single value"
                                );
                                return Err(ExitCode::from(2));
                            }
                            values.push((name, v.clone()));
                            i += 2;
                        }
                    }
                } else if a.starts_with("--") {
                    eprintln!("{cmd}: unknown option: {a}");
                    return Err(ExitCode::from(2));
                } else {
                    // A bare token. Counted, so `corpus sample --out /tmp/x` can
                    // no longer write into a directory literally named `--out`.
                    if positionals.len() == spec.max_positionals {
                        eprintln!(
                            "{cmd}: unexpected argument: {a} (this command takes {} positional \
                             argument(s))",
                            spec.max_positionals
                        );
                        return Err(ExitCode::from(2));
                    }
                    positionals.push(a.to_string());
                    i += 1;
                }
            }

            for &(dependent, required) in spec.requires {
                let have_dep = values.iter().any(|(n, _)| *n == dependent)
                    || flags.contains(&dependent);
                let have_req = values.iter().any(|(n, _)| *n == required)
                    || flags.contains(&required);
                if have_dep && !have_req {
                    eprintln!(
                        "{cmd}: {dependent} has no effect without {required}; refusing rather \
                         than dropping it silently"
                    );
                    return Err(ExitCode::from(2));
                }
            }

            Ok(Args { cmd, positionals, values, flags })
        }

        pub fn cmd(&self) -> &'static str {
            self.cmd
        }
        pub fn positionals(&self) -> &[String] {
            &self.positionals
        }
        pub fn first(&self) -> Option<&str> {
            self.positionals.first().map(String::as_str)
        }
        pub fn has(&self, name: &str) -> bool {
            self.flags.contains(&name)
        }
        pub fn get(&self, name: &str) -> Option<&str> {
            self.values.iter().find(|(n, _)| *n == name).map(|(_, v)| v.as_str())
        }
        pub fn all(&self, name: &str) -> Vec<&str> {
            self.values
                .iter()
                .filter(|(n, _)| *n == name)
                .map(|(_, v)| v.as_str())
                .collect()
        }
        pub fn path(&self, name: &str) -> Option<PathBuf> {
            self.get(name).map(PathBuf::from)
        }
        /// The single positional, as a path. Callers used to write
        /// `rest.first().filter(|s| !s.starts_with("--"))` — a hand-rolled guard
        /// that four handlers had and three did not.
        pub fn path_positional(&self) -> Option<PathBuf> {
            self.first().map(PathBuf::from)
        }

        /// A numeric option, where **a value that does not parse is an error**.
        ///
        /// The scan-era spelling was `opt(rest, "--compiles").and_then(|s|
        /// s.parse().ok())`, which turns a typo into the default *silently* —
        /// `--compiles abc` doubled a search budget from 200 to 400 and said
        /// nothing. Refusing costs nothing a correct invocation notices.
        pub fn num<T: FromStr>(&self, name: &str) -> Result<Option<T>, ExitCode> {
            match self.get(name) {
                None => Ok(None),
                Some(v) => match v.parse::<T>() {
                    Ok(n) => Ok(Some(n)),
                    Err(_) => {
                        eprintln!("{}: {name} expects a number, got {v:?}", self.cmd);
                        Err(ExitCode::from(2))
                    }
                },
            }
        }

        /// An option whose value must be one of a fixed set. A `_ =>` arm that
        /// swallows every unrecognised spelling is the dropped-flag failure in
        /// another costume: `retrieve eval --split heldout` ran leave-one-out's
        /// *opposite* and reported nothing.
        pub fn one_of<'a>(
            &'a self,
            name: &str,
            allowed: &[&'static str],
        ) -> Result<Option<&'a str>, ExitCode> {
            match self.get(name) {
                None => Ok(None),
                Some(v) if allowed.contains(&v) => Ok(Some(v)),
                Some(v) => {
                    eprintln!(
                        "{}: {name} expects one of {}, got {v:?}",
                        self.cmd,
                        allowed.join(" | ")
                    );
                    Err(ExitCode::from(2))
                }
            }
        }

        /// Locate the toolchain, or print the standard skip line and return
        /// `None` (the caller then exits SUCCESS).
        ///
        /// **This is the only producer of a `Toolchain` in this binary**, and it
        /// takes `&self` — so it is unreachable until the arguments have been
        /// parsed and validated. That is the whole point of the module; see the
        /// type-level docs.
        pub fn toolchain(&self) -> Option<Toolchain> {
            match Toolchain::locate() {
                Some(tc) => Some(tc),
                None => {
                    println!("SKIP: toolchain absent");
                    None
                }
            }
        }

        /// [`Args::toolchain`] without the `SKIP` line, for the one caller that
        /// reports absence *inside its own output* (`prefilter` emits a JSON
        /// verdict and must not interleave a bare line into it).
        pub fn toolchain_quiet(&self) -> Option<Toolchain> {
            Toolchain::locate()
        }
    }
}

use argv::{Args, Arity, Spec};

/// The `<cpp>` positional, from a **parsed** argument set.
///
/// It used to take `rest` and return `rest.first()` verbatim, which meant a
/// flag-shaped first token became the source path: `c2rs diff --help` looked for
/// a file called `--help`. `Args` has already separated options from
/// positionals, so that spelling is not expressible here any more.
fn require_cpp(args: &Args) -> Option<PathBuf> {
    match args.first() {
        Some(p) => Some(PathBuf::from(p)),
        None => {
            eprintln!("{}: expected a <cpp> path", args.cmd());
            None
        }
    }
}

/// `c2rs census <cpp>` — **P2b, single TU**: capture the bundle and print the
/// per-function verdict (modeled shape, or the first blocking feature).
///
/// The whole-TU verdict (`c2rs diff`) is all-or-nothing by design — the port
/// emits a complete obj or nothing — so it says only "NotImplemented" for a TU
/// where 99 of 100 functions are in class. This is the per-function view used
/// while developing a widening step: run it before and after, watch specific
/// functions move from a blocking feature to a shape.
/// Hex-dump a census blocking window, bracketing the byte that blocked the
/// parse: `b9 8b 0a >86< 43 9d 20`. The bracket is what makes the dump usable
/// without counting columns.
fn hexdump_marked(bytes: &[u8], mark: usize) -> String {
    let mut s = String::with_capacity(bytes.len() * 3 + 2);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        if i == mark {
            s.push_str(&format!(">{b:02x}<"));
        } else {
            s.push_str(&format!("{b:02x}"));
        }
    }
    s
}

/// The profile plumbing `capture`, `compile` and `census` share, plus the
/// `--cwd` dependency that all three used to drop in silence.
const CPP_PROFILE_REQUIRES: &[(&str, &str)] = &[("--cwd", "--flags-file")];

static CENSUS_SPEC: Spec = Spec::new(
    "census",
    &[
        ("--keep-il", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

fn cmd_census(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CENSUS_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    // Optional real-project capture (same inputs as `c2rs gap`), so a census can
    // be taken of an actual workload TU and not just an include-free fixture.
    // Keep the captured bundle for grammar work (gitignored scratch).
    let keep_il = args.path("--keep-il");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE the toolchain is located — the
    // ordering `capture` and `compile` already had and this command did not.
    // `census` read its `--flags-file` *inside* the post-`located()` capture
    // block, so `census x.cpp --flags-file /nonexistent` exited **0** with
    // `SKIP: toolchain absent` on a machine with no compilers, which is exactly
    // where the portable test lane runs. A usage error the binary never reports
    // is a usage error no test can pin.
    let flags: Vec<String> = match &flags_file {
        None => Vec::new(),
        Some(ff) => match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if flags_file.is_some() && flags.is_empty() {
        // `capture` and `compile` both refuse this; `census` did not, so an
        // all-comment flags file silently fell back to `cl.exe`'s own defaults
        // and the `/Gy`-dependent cross-check below was reported against a
        // profile nobody named.
        eprintln!("--flags-file names no flags; refusing to census at an unknown profile");
        return ExitCode::from(2);
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("census");
    // Two of the port's per-function refusals are `/Gy`-only, so the cross-check
    // below has to see the same flag the emitter would. The default capture is
    // `/Ox`, which does not imply it; a `--flags-file` may.
    let mut gy = false;
    // Print the profile that was actually used, always — the affordance
    // `capture` and `compile` have and this command lacked, which is why its
    // dropped `--cwd` had no terminal signal at all.
    match &flags_file {
        None => println!(
            "  profile: {} (default — NOT the workload's; /Ox does not imply /GF)",
            c2_reference::CAPTURE_IL_DEFAULT_FLAGS.join(" ")
        ),
        Some(ff) => println!("  profile: {} (from {})", flags.join(" "), ff.display()),
    }
    if let Some(d) = &cwd {
        println!("  cwd:     {}", d.display());
    }
    let captured = match &flags_file {
        None => tc.capture_il(&cpp, &w),
        Some(_) => {
            gy = c2_core::PortC2::flags_imply_function_level_linking(&flags);
            tc.capture_reference_with(&cpp.to_string_lossy(), &w, &flags, cwd.as_deref())
                .map(|c| c.bundle)
        }
    };
    let bundle = match captured {
        Ok(b) => b,
        Err(e) => {
            eprintln!("capture failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    if let Some(dir) = &keep_il {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create --keep-il {}: {e}", dir.display());
        } else {
            for suffix in IL_SUFFIXES {
                if let Some(bytes) = bundle.get(suffix) {
                    let p = dir.join(format!("{}.{suffix}", bundle.base_name));
                    if let Err(e) = std::fs::write(&p, bytes) {
                        eprintln!("cannot write {}: {e}", p.display());
                    }
                }
            }
            println!("kept IL bundle in {}", dir.display());
        }
    }
    let Some(rows) = bundle.census_functions() else {
        eprintln!("census unavailable: bundle is missing .ex/.gl");
        let _ = std::fs::remove_dir_all(&w);
        return ExitCode::FAILURE;
    };
    // The census/gate cross-check, per TU (roadmap #44): a function the census
    // calls in class that `PortC2`'s own selector refuses. `c2rs census` is the
    // instrument a widening step is developed against, so it has to show this —
    // `int f(int a,int b,int c){ return a + b*c; }` read `1/1 in class` beside a
    // `Port=NotImplemented` for as long as the disagreement existed.
    let mut gate_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for (f, gate) in &rows {
        if !f.verdict.in_class() {
            continue;
        }
        let key = match gate {
            Err(e) => Some((*e).to_string()),
            Ok(func) => match c2_core::codegen::opt_mode_of_word(f.opt_word) {
                Err(e) => Some(e.to_string()),
                Ok(mode) => c2_core::codegen::function_gate(func, mode, gy)
                    .err()
                    .map(|e| e.to_string()),
            },
        };
        if let Some(k) = key {
            *gate_hist.entry(k).or_insert(0) += 1;
        }
    }
    // The `.gl` binding invariants (D14), per TU: what every generated empty
    // destructor resolved to, and whether any token is claimed twice. The oracle
    // cannot grade a correspondence, so these are printed where a widening step is
    // developed, not only in the scan aggregate (`docs/GAPS.md` §6).
    let mut dtor_callees: Vec<(String, String)> = Vec::new();
    for (f, gate) in &rows {
        if f.verdict.key().starts_with("empty-dtor") {
            if let Ok(func) = gate {
                dtor_callees.push((
                    f.name.clone().unwrap_or_else(|| format!("#{}", f.index)),
                    func.tail_call.clone().unwrap_or_default(),
                ));
            }
        }
    }
    let (gl_dropped, gl_conflicts) = bundle
        .get("gl")
        .map(|g| c2_il::gl_symbol_conflicts(g))
        .unwrap_or((0, 0));
    let census: Vec<c2_il::FnCensus> = rows.into_iter().map(|(c, _)| c).collect();
    let in_class = census.iter().filter(|f| f.verdict.in_class()).count();
    println!(
        "{} -> {}/{} functions in class",
        cpp.display(),
        in_class,
        census.len()
    );
    if gl_dropped > 0 {
        println!(
            "  .gl ambiguous tokens dropped: {gl_dropped} ({gl_conflicts} involving a mangled \
             name — that count must be 0)"
        );
    }
    if !dtor_callees.is_empty() {
        let bad = dtor_callees
            .iter()
            .filter(|(_, c)| c2_harness::gap::dtor_callee_class(c) == "other")
            .count();
        println!(
            "  generated empty destructors: {} bound, {bad} to a NON-destructor",
            dtor_callees.len()
        );
        for (f, c) in dtor_callees.iter().take(12) {
            println!("    {f} -> {c}");
        }
    }
    let mut hist: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    // One representative blocking-site hexdump per feature, so a big TU reports
    // each distinct gap once instead of thousands of times.
    let mut sample: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    // The control-flow axis, over EVERY function including the in-class ones —
    // they are the control group, and every one of them must read
    // `cflow-straight`, because every shape the port accepts is a single basic
    // block. A `cflow-loop` among them would indict the measure.
    let mut cflow_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    // The EH axis beside it, likewise over every function. Here the in-class
    // rows are more than a control group: the three `empty-dtor-*` shapes ARE
    // the cheap side of `docs/EH_RECORDS.md` §6's boundary, so one of them
    // reading anything but `eh-bare` indicts the axis.
    let mut eh_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    // Per-function lines are only readable for a small TU; a real one has
    // thousands of functions and the histogram is the useful view.
    let list_each = census.len() <= 64;
    for f in &census {
        let mark = if f.verdict.in_class() { "ok " } else { "GAP" };
        if list_each {
            // Both census axes on one line. A control-flow fixture is graded on
            // the pair: it must refuse (the first column) AND its shape must be
            // decoded (the second), and a single column can show only one of
            // those. `c2rs gap` prints the same second axis aggregated.
            println!(
                "  [{:>3}] {mark} {:<24} {:<26} {:<11} ({:<12}) {:>6} B  {}",
                f.index,
                f.verdict.key(),
                f.cflow,
                f.eh,
                f.eh_stmt,
                f.seg_len,
                f.name.as_deref().unwrap_or("(unnamed)")
            );
        }
        *cflow_hist.entry(f.cflow.clone()).or_insert(0) += 1;
        *eh_hist.entry(f.eh.clone()).or_insert(0) += 1;
        if !f.verdict.in_class() {
            *hist.entry(f.verdict.key()).or_insert(0) += 1;
            sample
                .entry(f.verdict.key())
                .or_insert_with(|| hexdump_marked(&f.hex, f.hex_mark));
        }
    }
    if !hist.is_empty() {
        let mut v: Vec<_> = hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        println!("  blocking features (\">\" marks the byte that blocked the parse):");
        for (feature, count) in v.iter().take(24) {
            println!("    {count:>6} x {feature}");
            if let Some(h) = sample.get(feature) {
                println!("             {h}");
            }
        }
        if v.len() > 24 {
            println!("    … and {} more distinct features", v.len() - 24);
        }
    }
    {
        let mut v: Vec<_> = cflow_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let decoded: usize = v.iter().filter(|(k, _)| k.starts_with("cflow-")).map(|(_, n)| n).sum();
        println!(
            "  control-flow class (decode-only): {decoded}/{} bodies decoded end to end",
            census.len()
        );
        for (class, count) in v.iter().take(16) {
            println!("    {count:>6} x {class}");
        }
    }
    {
        let mut v: Vec<_> = eh_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let need: usize = v
            .iter()
            .filter(|(k, _)| k.as_str() == "eh-state1")
            .map(|(_, n)| n)
            .sum();
        println!(
            "  EH class (maxState, decode-only): {need}/{} bodies have maxState >= 1 and need the \
             whole EH record",
            census.len()
        );
        for (class, count) in v.iter() {
            println!("    {count:>6} x {class}");
        }
    }
    if !gate_hist.is_empty() {
        let n: usize = gate_hist.values().sum();
        println!(
            "  census/gate DISAGREEMENT: {n} of the {in_class} in class are refused by PortC2:"
        );
        let mut v: Vec<_> = gate_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (reason, count) in v.iter().take(12) {
            println!("    {count:>6} x {reason}");
        }
    }
    let _ = std::fs::remove_dir_all(&w);
    ExitCode::SUCCESS
}

static CAPTURE_SPEC: Spec = Spec::new(
    "capture",
    &[
        // `--keep-il DIR` retains the captured bundle for byte inspection — the
        // same affordance `compile --keep-obj` gives for the reference obj, and
        // the only way to design a fixture around a *record-level* `.gl` shape
        // (which name separator introduces a run, where a record's framing
        // starts) without guessing. Gitignored scratch only.
        ("--keep-il", Arity::Value),
        // `--flags-file` / `--cwd`: without them every `.gl` captured for
        // analysis was taken at the `/Ox /GS- /c` default while the obj it was
        // read against had been compiled at the workload's `/O1 /Oi /EHsc /GR …`
        // — and `/Ox` does not imply `/GF`, which is exactly the skew
        // `gl_string_comdat_names` exists to catch.
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

fn cmd_capture(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CAPTURE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let keep_il = args.path("--keep-il");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE the toolchain is located, so a
    // malformed invocation is reported as one on a machine with no compilers at
    // all. Only the capture itself needs the toolchain, and that still degrades
    // to a clean exit 0.
    //
    // No `--flags-file` keeps the default byte-for-byte: `capture_il` is still
    // the call, with `CAPTURE_IL_DEFAULT_FLAGS`. This is a widening.
    let flags: Vec<String> = match &flags_file {
        None => Vec::new(),
        Some(ff) => match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if flags_file.is_some() && flags.is_empty() {
        // An empty profile would silently fall back to `cl.exe`'s own defaults —
        // the dropped-flag failure mode again, one layer down.
        eprintln!("--flags-file names no flags; refusing to capture at an unknown profile");
        return ExitCode::from(2);
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("capture");
    let captured = match &flags_file {
        None => tc.capture_il(&cpp, &w),
        Some(_) => tc.capture_il_flags(&cpp, &w, &flags, cwd.as_deref()),
    };
    match captured {
        Ok(bundle) => {
            println!("captured IL bundle {} from {}", bundle.base_name, cpp.display());
            // Print the profile that was actually used, always. A flag that is
            // dropped in silence is indistinguishable from a flag that had no
            // effect, and this line is what tells the two apart at the terminal.
            match &flags_file {
                None => println!(
                    "  profile: {} (default — NOT the workload's; /Ox does not imply /GF)",
                    c2_reference::CAPTURE_IL_DEFAULT_FLAGS.join(" ")
                ),
                Some(ff) => println!("  profile: {} (from {})", flags.join(" "), ff.display()),
            }
            if let Some(d) = &cwd {
                println!("  cwd:     {}", d.display());
            }
            for suffix in IL_SUFFIXES {
                let size = bundle.get(suffix).map(|b| b.len()).unwrap_or(0);
                let present = if bundle.get(suffix).is_some() { "ok" } else { "MISSING" };
                println!("  .{suffix:<2}  {size:>7} B  {present}");
            }
            if let Some(dir) = &keep_il {
                let _ = std::fs::create_dir_all(dir);
                for suffix in IL_SUFFIXES {
                    if let Some(bytes) = bundle.get(suffix) {
                        let p = dir.join(format!("{}.{suffix}", bundle.base_name));
                        match std::fs::write(&p, bytes) {
                            Ok(()) => println!("  kept {}", p.display()),
                            Err(e) => eprintln!("  keep-il {} failed: {e}", p.display()),
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("capture failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::FAILURE
        }
    }
}

static COMPILE_SPEC: Spec = Spec::new(
    "compile",
    &[
        // `--keep-obj PATH` retains the reference obj for byte classification
        // (the CONST/DERIVED analysis every widening step starts from).
        ("--keep-obj", Arity::Value),
        // Optional real-project compile (same inputs as `c2rs gap`), so the
        // reference obj for a workload TU can be classified, not just a
        // fixture's.
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

fn cmd_compile(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&COMPILE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let keep_obj = args.path("--keep-obj");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE `located()`, so a malformed
    // invocation is reported as one on a machine with no compilers at all —
    // which is what lets `tests/cli_flags.rs` catch this class without a
    // toolchain. A *valid* invocation still exits 0 with `SKIP: toolchain
    // absent`.
    let flags: Vec<String> = match &flags_file {
        None => Vec::new(),
        Some(ff) => match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if flags_file.is_some() && flags.is_empty() {
        // An empty profile would silently fall back to `cl.exe`'s own defaults —
        // the dropped-profile failure mode one layer down, and the reason
        // `cmd_capture` refuses it too.
        eprintln!("--flags-file names no flags; refusing to compile at an unknown profile");
        return ExitCode::from(2);
    }
    // The `--cwd`-without-`--flags-file` refusal is no longer written here: it
    // is a `requires` edge on the spec, shared by `capture`, `census` and
    // `compile`. Three commands had the same dangling option and only one
    // refused it, which is what a rule expressed in prose rather than in one
    // place gets you.
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("compile");
    let out = w.join("out.obj");
    if let Some(ff) = &flags_file {
        let res = tc.capture_reference_with(&cpp.to_string_lossy(), &w, &flags, cwd.as_deref());
        return match res {
            Ok(c) => {
                println!(
                    "compiled {} -> {} bytes (project flags)",
                    cpp.display(),
                    c.ref_obj.len()
                );
                // Print the profile that was actually used, always. A flag
                // dropped in silence is indistinguishable at the terminal from a
                // flag that had no effect, and this line is what tells them
                // apart.
                println!("  profile: {} (from {})", flags.join(" "), ff.display());
                if let Some(d) = &cwd {
                    println!("  cwd:     {}", d.display());
                }
                if let Some(dest) = &keep_obj {
                    if let Some(p) = dest.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::write(dest, c.ref_obj.as_bytes());
                    println!("  kept reference obj at {}", dest.display());
                }
                let _ = std::fs::remove_dir_all(&w);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("compile failed: {e}");
                let _ = std::fs::remove_dir_all(&w);
                ExitCode::FAILURE
            }
        };
    }
    match tc.compile_obj(&cpp, &out) {
        Ok(obj) => {
            let ts = obj
                .timestamp()
                .map(|t| format!("0x{t:08x}"))
                .unwrap_or_else(|| "<none>".to_string());
            println!(
                "compiled {} -> {} bytes, TimeDateStamp={}",
                cpp.display(),
                obj.len(),
                ts
            );
            // `Toolchain::compile_obj` hard-codes the same three flags
            // `capture_il` does, which is why the published constant is what is
            // printed here rather than a second literal — one place names them.
            println!(
                "  profile: {} (default — NOT the workload's; /Ox does not imply /GF)",
                c2_reference::CAPTURE_IL_DEFAULT_FLAGS.join(" ")
            );
            if let Some(dest) = &keep_obj {
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(dest, obj.as_bytes()) {
                    Ok(()) => println!("  kept reference obj at {}", dest.display()),
                    Err(e) => eprintln!("  cannot write {}: {e}", dest.display()),
                }
            }
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("compile failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::FAILURE
        }
    }
}

fn selftest_row(r: &SelfTestReport) -> String {
    let name = r
        .cpp
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| r.cpp.display().to_string());
    let detail = match &r.outcome {
        SelfTestOutcome::Pass { obj_len, ex_len } => {
            format!("PASS   obj={obj_len}B ex={ex_len}B")
        }
        SelfTestOutcome::DeterminismFail {
            first_offset,
            len_a,
            len_b,
        } => format!("FAIL   determinism @off {first_offset} (len {len_a} vs {len_b})"),
        SelfTestOutcome::CaptureUnstable { ex_len_a, ex_len_b } => {
            format!("FAIL   capture-unstable (.ex {ex_len_a}B vs {ex_len_b}B)")
        }
        SelfTestOutcome::Error(msg) => format!("ERROR  {}", first_line(msg)),
    };
    format!("  {name:<34} {detail}")
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

/// `selftest` takes any number of `<cpp>` positionals and no options. It used to
/// map `rest` wholesale to fixture paths, so `c2rs selftest --flags-file f.txt`
/// looked for two "fixtures" named `--flags-file` and `f.txt` and failed as a
/// missing file — exit 1, not a usage error.
static SELFTEST_SPEC: Spec = Spec::new("selftest", &[]).positionals(usize::MAX);

fn cmd_selftest(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SELFTEST_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let targets: Vec<PathBuf> = if args.positionals().is_empty() {
        all_fixtures()
    } else {
        args.positionals().iter().map(PathBuf::from).collect()
    };
    if targets.is_empty() {
        eprintln!("no fixtures found");
        return ExitCode::FAILURE;
    }
    let mut all_pass = true;
    // The oracle self-test IS the correctness benchmark, so it names the oracle
    // it ran against (roadmap #48): a stale wibo turns this seam's verdicts over
    // without changing any other number in the report.
    print!("{}", Provenance::collect(&tc, None).render());
    println!("oracle self-test (determinism + capture stability):");
    for cpp in &targets {
        let w = scratch("selftest");
        let report = oracle_selftest(cpp, &tc, &w);
        all_pass &= report.passed();
        println!("{}", selftest_row(&report));
        let _ = std::fs::remove_dir_all(&w);
    }
    if all_pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs replay <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`replay` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
static REPLAY_SPEC: Spec = Spec::new("replay", &[]);

fn cmd_replay(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&REPLAY_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    if !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return ExitCode::SUCCESS;
    }
    let w = scratch("replay");
    let out = (|| {
        let captured = tc.capture_reference(&cpp, &w.join("cap"))?;
        // Replay to the SAME /Fo path as the reference for an exact byte compare.
        let ref_path = captured.ref_obj_path.clone();
        let replay = tc.replay(&captured, &w.join("replay_il"), &ref_path)?;
        Ok::<_, std::io::Error>((captured, replay))
    })();
    let code = match out {
        Ok((captured, replay)) => {
            let raw = captured.ref_obj.as_bytes() == replay.as_bytes();
            let norm = matches!(
                ObjImage::diff(&captured.ref_obj, &replay),
                ObjDiff::Identical
            );
            println!(
                "{} -> ref={}B replay={}B  raw_identical={raw}  normalized_identical={norm}",
                cpp.display(),
                captured.ref_obj.len(),
                replay.len(),
            );
            if norm {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("replay failed: {e}");
            ExitCode::FAILURE
        }
    };
    let _ = std::fs::remove_dir_all(&w);
    code
}

/// P-F0.1: capture the IL bundle, then reproduce it by driving `c1xx.dll` alone
/// (the front-end analogue of `replay`). Prints a per-file byte verdict; exits
/// non-zero only when a present file failed to reproduce byte-for-byte (a real
/// failure of the front-end replay oracle) or the capture/replay errored.
/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs replay-c1 <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`replay-c1` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
static REPLAY_C1_SPEC: Spec = Spec::new("replay-c1", &[]);

fn cmd_replay_c1(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&REPLAY_C1_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed to build the c1host stub)");
        return ExitCode::SUCCESS;
    }
    if !tc.has_c1xx() {
        println!("SKIP: c1xx.dll absent (front end not located)");
        return ExitCode::SUCCESS;
    }
    let w = scratch("replay-c1");
    let report = c1_replay_check(&cpp, &tc, &w);
    let code = match &report {
        C1ReplayReport::ToolchainAbsent => {
            println!("SKIP: toolchain absent");
            ExitCode::SUCCESS
        }
        C1ReplayReport::Skipped(msg) => {
            println!("SKIP: {}", first_line(msg));
            ExitCode::SUCCESS
        }
        C1ReplayReport::ReferenceError(msg) => {
            eprintln!("c1 replay error: {}", first_line(msg));
            ExitCode::FAILURE
        }
        C1ReplayReport::Replayed { base, files } => {
            let all = report.all_identical();
            println!(
                "{} -> front-end bundle {base}  {}",
                cpp.display(),
                if all { "REPRODUCED byte-exact" } else { "DIVERGED" }
            );
            for f in files {
                let verdict = if f.identical {
                    "identical".to_string()
                } else {
                    format!(
                        "DIFFERS @ {} (cap={}B replay={}B)",
                        f.first_offset.unwrap_or(0),
                        f.cap_len,
                        f.replay_len
                    )
                };
                println!("  .{:<2}  {:>7} B  {verdict}", f.suffix, f.cap_len);
            }
            if all {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    };
    let _ = std::fs::remove_dir_all(&w);
    code
}

/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs diff <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`diff` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
static DIFF_SPEC: Spec = Spec::new("diff", &[]);

fn cmd_diff(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&DIFF_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("diff");
    let port = PortC2::default();
    let report = differential(&cpp, &tc, &port, &w);
    let line = match &report {
        DiffReport::ToolchainAbsent => "ToolchainAbsent".to_string(),
        DiffReport::Skipped(msg) => format!("SKIP: {}", first_line(msg)),
        DiffReport::ReferenceError(msg) => format!("ReferenceError: {}", first_line(msg)),
        DiffReport::ReferenceReplayMismatch {
            first_offset,
            ref_len,
            replay_len,
        } => format!(
            "ReferenceReplay=MISMATCH @ offset {first_offset} (ref={ref_len}B replay={replay_len}B)"
        ),
        DiffReport::ReferenceReplayByteExact {
            ref_len,
            replay_len,
            port,
        } => {
            let port_str = match port {
                PortStatus::NotImplemented(_) => "NotImplemented".to_string(),
                PortStatus::Match => "Match".to_string(),
                PortStatus::Mismatch { first_offset } => {
                    format!("Mismatch @ offset {first_offset}")
                }
            };
            format!(
                "ReferenceReplay=ByteExact (ref={ref_len}B replay={replay_len}B)  Port={port_str}"
            )
        }
    };
    println!("{} -> {}", cpp.display(), line);
    let _ = std::fs::remove_dir_all(&w);
    // A byte-exact reference replay is the pass condition, and the port may be
    // Match or NotImplemented depending on the TU — both, and clean skips, are
    // success for scripting.
    //
    // `Port=Mismatch` is NOT. The doctrine is that a mismatch is an alarm rather
    // than a gap: the port emitted bytes and they were wrong. This is the per-rung
    // acceptance gate, so the alarm needs an exit code and not just a line of
    // stdout that a `tail -1` may or may not be read by a human.
    match &report {
        DiffReport::ReferenceReplayMismatch { .. } | DiffReport::ReferenceError(_) => {
            ExitCode::FAILURE
        }
        DiffReport::ReferenceReplayByteExact { port: PortStatus::Mismatch { .. }, .. } => {
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}

/// `bench` takes nothing. The dispatcher used to call it as `cmd_bench()`, so
/// every argument after `bench` was discarded **by the dispatcher**, one level
/// above any handler that could have refused it — the same class, at the only
/// site where the handler never even saw the arguments.
static BENCH_SPEC: Spec = Spec::new("bench", &[]).positionals(0);

fn cmd_bench(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&BENCH_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let targets = all_fixtures();
    if targets.is_empty() {
        eprintln!("no fixtures found under {}", c2_harness::fixtures_dir().display());
        return ExitCode::FAILURE;
    }
    println!("bench: oracle self-test across {} fixture(s)", targets.len());
    let (mut pass, mut fail, mut err) = (0u32, 0u32, 0u32);
    for cpp in &targets {
        let w = scratch("bench");
        let report = oracle_selftest(cpp, &tc, &w);
        match &report.outcome {
            SelfTestOutcome::Pass { .. } => pass += 1,
            SelfTestOutcome::Error(_) => err += 1,
            _ => fail += 1,
        }
        println!("{}", selftest_row(&report));
        let _ = std::fs::remove_dir_all(&w);
    }
    println!("\nsummary: {pass} pass, {fail} fail, {err} error (of {})", targets.len());
    if fail == 0 && err == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ---------------------------------------------------------------------------
// perf — angle-H latency: native port vs standalone c2 (IL bundle -> obj)
// ---------------------------------------------------------------------------

use c2_harness::perf::{self, fmt_dur, PerfConfig, PortPerf};

static PERF_SPEC: Spec = Spec::new(
    "perf",
    &[
        ("--port-iters", Arity::Value),
        ("--ref-iters", Arity::Value),
        ("--fixtures", Arity::Value),
    ],
)
.positionals(0);

fn cmd_perf(rest: &[String]) -> ExitCode {
    // Parse and validate FIRST. This handler used to call `located()` as its
    // opening statement, so on a machine with no compilers `c2rs perf --typo`
    // exited **0** with `SKIP: toolchain absent` and the typo was never
    // reported. That is the ordering half of the class, and it is now
    // inexpressible: `args.toolchain()` needs an `args`.
    let args = match Args::parse(&PERF_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let mut cfg = PerfConfig::default();
    // `.parse().ok()` turned a typo into the default in silence. `num` refuses.
    match args.num::<usize>("--port-iters") {
        Ok(Some(v)) => cfg.port_iters = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<usize>("--ref-iters") {
        Ok(Some(v)) => cfg.ref_iters = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for standalone-c2 replay)");
        return ExitCode::SUCCESS;
    }
    let targets: Vec<PathBuf> = match args.get("--fixtures") {
        Some(list) => list
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                let p = PathBuf::from(s);
                if p.exists() {
                    p
                } else {
                    c2_harness::fixtures_dir().join(s)
                }
            })
            .collect(),
        None => all_fixtures(),
    };
    if targets.is_empty() {
        eprintln!("no fixtures to benchmark");
        return ExitCode::FAILURE;
    }

    println!(
        "perf: IL-bundle -> obj latency, native port vs standalone c2 (reference)\n\
         \x20 {} fixture(s), port_iters={}, ref_iters={}   (both produce the SAME obj)\n",
        targets.len(),
        cfg.port_iters,
        cfg.ref_iters,
    );
    println!(
        "  {:<28} {:>7}  {:>13}  {:>13}  {:>11}  {}",
        "fixture", "obj", "ref median", "port median", "speedup", "port"
    );

    let mut rows = Vec::new();
    let mut errors = 0usize;
    for cpp in &targets {
        let name = cpp
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| cpp.display().to_string());
        let w = scratch("perf");
        match perf::bench_fixture(&tc, cpp, &cfg, &w) {
            Ok(r) => {
                let (port_med, speedup, status) = match r.port {
                    PortPerf::Match { median, .. } => (
                        fmt_dur(median),
                        r.speedup()
                            .map(|s| format!("{s:.0}x"))
                            .unwrap_or_else(|| "-".into()),
                        "Match".to_string(),
                    ),
                    PortPerf::NotImplemented => {
                        ("-".into(), "-".into(), "NotImplemented".to_string())
                    }
                    PortPerf::Mismatch { first_offset } => {
                        ("-".into(), "-".into(), format!("Mismatch@{first_offset}"))
                    }
                };
                // The P0.1 invariant should always hold; flag it loudly if not.
                let flag = if r.ref_exact { "" } else { "  [!ref-replay-inexact]" };
                println!(
                    "  {:<28} {:>6}B  {:>13}  {:>13}  {:>11}  {}{}",
                    name,
                    r.obj_len,
                    fmt_dur(r.ref_median),
                    port_med,
                    speedup,
                    status,
                    flag,
                );
                rows.push(r);
            }
            Err(e) => {
                println!("  {name:<28} ERROR {}", first_line(&e.to_string()));
                errors += 1;
            }
        }
        let _ = std::fs::remove_dir_all(&w);
    }

    let report = perf::PerfReport {
        rows,
        port_iters: cfg.port_iters,
        ref_iters: cfg.ref_iters,
    };
    let (matched, mismatched, ni) = report.tally();
    let ref_inexact = report.rows.iter().filter(|r| !r.ref_exact).count();
    println!(
        "\nsummary: {matched} port Match, {mismatched} mismatch, {ni} not-implemented (of {})",
        report.rows.len()
    );
    match report.geomean_speedup() {
        Some(g) => println!(
            "  geomean speedup over the {matched} matched fixture(s): {g:.0}x faster than standalone c2"
        ),
        None => println!("  no matched fixtures — no speedup to report"),
    }
    // Convention (as in `diff`): the reference is the sole judge, so a port
    // Match/Mismatch/NotImplemented is per-TU reporting, not a harness failure.
    // Only a capture/replay error or a broken P0.1 replay (ref-replay-inexact)
    // is a hard failure of the benchmark itself.
    if errors > 0 || ref_inexact > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn default_concurrencies() -> Vec<usize> {
    // Powers of two up to the machine's parallelism (capped at 32 for the graph).
    let max = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
        .min(32);
    let mut v = Vec::new();
    let mut c = 1;
    while c <= max {
        v.push(c);
        c *= 2;
    }
    if *v.last().unwrap_or(&0) != max {
        v.push(max);
    }
    v
}

static PERF_SCALE_SPEC: Spec = Spec::new(
    "perf-scale",
    &[
        ("--fixture", Arity::Value),
        ("--conc", Arity::Value),
        ("--port-secs", Arity::Value),
        ("--ref-secs", Arity::Value),
        ("--csv", Arity::Value),
    ],
)
.positionals(0);

fn cmd_perf_scale(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&PERF_SCALE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let fixture = args.get("--fixture")
        .map(|s| {
            let p = PathBuf::from(s);
            if p.exists() {
                p
            } else {
                c2_harness::fixtures_dir().join(s)
            }
        })
        .unwrap_or_else(|| c2_harness::fixtures_dir().join("mvp_add3.cpp"));

    let mut cfg = perf::ScaleConfig::default();
    // `filter_map(parse().ok())` SILENTLY DROPPED a bad element: `--conc 1,x,4`
    // ran `[1, 4]` and printed `concurrencies=[1, 4]` as if that were what was
    // asked for. Only an all-bad list reached the refusal below, so the failure
    // was invisible exactly when it was partial.
    cfg.concurrencies = match args.get("--conc") {
        Some(list) => {
            let mut v: Vec<usize> = Vec::new();
            for tok in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                match tok.parse::<usize>() {
                    Ok(c) if c >= 1 => v.push(c),
                    _ => {
                        eprintln!("perf-scale: --conc expects positive integers, got {tok:?}");
                        return ExitCode::from(2);
                    }
                }
            }
            v
        }
        None => default_concurrencies(),
    };
    if cfg.concurrencies.is_empty() {
        eprintln!("no valid --conc values");
        return ExitCode::from(2);
    }
    match args.num::<f64>("--port-secs") {
        Ok(Some(v)) => cfg.port_secs = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<f64>("--ref-secs") {
        Ok(Some(v)) => cfg.ref_secs = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for standalone-c2 replay)");
        return ExitCode::SUCCESS;
    }

    let name = fixture
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| fixture.display().to_string());
    println!(
        "perf-scale: throughput (objs/sec) vs concurrency on {name}\n\
         \x20 concurrencies={:?}  port_secs={}  ref_secs={}\n",
        cfg.concurrencies, cfg.port_secs, cfg.ref_secs
    );

    let w = scratch("perf-scale");
    let (points, obj_len) = match perf::scale_measure(&tc, &fixture, &cfg, &w) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("perf-scale failed: {}", first_line(&e.to_string()));
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    let _ = std::fs::remove_dir_all(&w);

    println!("  obj size: {obj_len} B (both sides produce this exact obj)\n");
    println!(
        "  {:>5}  {:>16}  {:>16}  {:>10}",
        "conc", "port objs/sec", "c2 objs/sec", "speedup"
    );
    for p in &points {
        println!(
            "  {:>5}  {:>16.0}  {:>16.1}  {:>9.0}x",
            p.concurrency,
            p.port_ops,
            p.ref_ops,
            p.speedup()
        );
    }

    // Emit CSV for the README plot when asked.
    if let Some(path) = args.get("--csv") {
        let mut csv = String::from("concurrency,port_ops_per_sec,ref_ops_per_sec\n");
        for p in &points {
            csv.push_str(&format!("{},{:.3},{:.3}\n", p.concurrency, p.port_ops, p.ref_ops));
        }
        match std::fs::write(path, csv) {
            Ok(()) => println!("\nwrote CSV: {path}"),
            Err(e) => {
                eprintln!("could not write CSV {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

/// **Board #132 — the listing seam.** Capture one TU and print (or write) c2's
/// own `.cod` assembly listing beside the obj the differential grades.
///
/// The listing is a **decode aid, never a gate**: the obj byte-compare remains
/// the sole judge of the port.
static LISTING_SPEC: Spec = Spec::new(
    "listing",
    &[
        ("--qxstalls", Arity::Flag),
        ("--out", Arity::Value),
        // Repeatable: `--flag /GR- --flag /Ox` builds the profile.
        ("--flag", Arity::Repeated),
    ],
);

fn cmd_listing(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&LISTING_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let out = args.path("--out");
    let qxstalls = args.has("--qxstalls");
    let mut flags: Vec<String> = args.all("--flag").into_iter().map(String::from).collect();
    let Some(cpp) = args.first().map(PathBuf::from) else {
        eprintln!(
            "usage: c2rs listing <cpp> [--qxstalls] [--out PATH] [--flag F ...]\n\
             default flags: /O1 /Oi /EHsc /GS- /c"
        );
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    if flags.is_empty() {
        flags = ["/O1", "/Oi", "/EHsc", "/GS-", "/c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    }
    let w = scratch("listing");
    let src = c2_reference::to_wibo_path(&cpp);
    let code = match tc.capture_listing_with(&src, &w, &flags, None, qxstalls) {
        Ok((captured, cod)) => {
            let listing = c2_reference::cod::CodListing::parse(&cod);
            let emitted = captured
                .ref_obj
                .text_comdat_functions()
                .map(|v| v.len())
                .unwrap_or(0);
            println!(
                "{} -> obj={}B  cod={}B  {} PROC / {} .text COMDAT / {} PUBLIC{}",
                cpp.display(),
                captured.ref_obj.len(),
                cod.len(),
                listing.functions.len(),
                emitted,
                listing.publics.len(),
                if qxstalls { "  [/QXSTALLS]" } else { "" },
            );
            match &out {
                Some(p) => match std::fs::write(p, cod.as_bytes()) {
                    Ok(()) => {
                        println!("  listing written to {}", p.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("cannot write {}: {e}", p.display());
                        ExitCode::FAILURE
                    }
                },
                None => {
                    print!("{cod}");
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            eprintln!("listing capture failed: {e}");
            ExitCode::FAILURE
        }
    };
    let _ = std::fs::remove_dir_all(&w);
    code
}

/// **Boards #134 and #136** — the population scan over the listing seam.
static LISTING_SCAN_SPEC: Spec = Spec::new(
    "listing-scan",
    &[
        ("--qxstalls", Arity::Flag),
        ("--list", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--limit", Arity::Value),
        ("--jobs", Arity::Value),
        ("--jsonl", Arity::Value),
        ("--work", Arity::Value),
    ],
)
.positionals(0);

fn cmd_listing_scan(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&LISTING_SCAN_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let cwd = args.path("--cwd");
    let jsonl = args.path("--jsonl");
    let work = args.path("--work");
    let qxstalls = args.has("--qxstalls");
    // `--limit`/`--jobs` used to `return ExitCode::from(2)` on an unparseable
    // value with NO message at all — the `{name} needs a value` line only fired
    // when the token was missing entirely, so `--jobs eight` exited 2 in
    // silence. `num` names the option and the value.
    let limit: Option<usize> = match args.num("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let jobs: usize = match args.num("--jobs") {
        Ok(Some(v)) => v,
        Ok(None) => std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        Err(c) => return c,
    };
    let (list_file, flags_file) = (args.path("--list"), args.path("--flags-file"));
    let (Some(list_file), Some(flags_file)) = (list_file, flags_file) else {
        eprintln!(
            "usage: c2rs listing-scan --list FILE --flags-file FILE [--cwd DIR] \
             [--limit N] [--jobs N] [--qxstalls] [--jsonl PATH] [--work DIR]"
        );
        return ExitCode::from(2);
    };
    let read_tokens = |p: &PathBuf, split: bool| -> std::io::Result<Vec<String>> {
        let text = std::fs::read_to_string(p)?;
        let mut out = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if split {
                out.extend(line.split_whitespace().map(String::from));
            } else {
                out.push(line.to_string());
            }
        }
        Ok(out)
    };
    let sources = match read_tokens(&list_file, false) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --list {}: {e}", list_file.display());
            return ExitCode::FAILURE;
        }
    };
    let flags = match read_tokens(&flags_file, true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --flags-file {}: {e}", flags_file.display());
            return ExitCode::FAILURE;
        }
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }

    let cfg = c2_harness::listing::ListingScanConfig {
        sources,
        flags,
        cwd,
        limit,
        jobs,
        work: work.unwrap_or_else(|| scratch("listing-scan")),
        qxstalls,
        jsonl,
    };
    let total_hint = cfg.limit.unwrap_or(cfg.sources.len());
    eprintln!(
        "listing-scan: {} TUs, {} jobs, /QXSTALLS {}",
        total_hint,
        cfg.jobs,
        if cfg.qxstalls { "ON" } else { "OFF (control)" }
    );
    let report = match c2_harness::listing::listing_scan(&tc, &cfg, &|n, total, r| {
        if n % 25 == 0 || n == total {
            eprintln!("  [{n}/{total}] {}{}", r.src, if r.error.is_empty() { "" } else { "  CAPTURE-FAIL" });
        }
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("listing scan failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    let pct = |a: usize, b: usize| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };

    println!("\nLISTING SCAN — {} TUs captured, {} failed", report.captured, report.failed);

    // ---- #136 -------------------------------------------------------------
    let (procs, comdats, dupes, cod_only, obj_only) = report.reconcile_totals();
    let (agree, be_total) = report.byte_exact_agreement();
    let (in_class, emitted, unbound) = report.emitted_census();
    println!("\n#136  THE TWO SOURCES, RECONCILED");
    println!("  .cod PROC set          {procs}");
    println!("  obj .text COMDAT set   {comdats}");
    println!(
        "  invariant 1 injectivity     duplicate PROC names: {dupes}  ({})",
        if dupes == 0 { "PASS" } else { "FAIL" }
    );
    println!(
        "  invariant 2 totality        cod-only {cod_only}, obj-only {obj_only}  ({})",
        if cod_only == 0 && obj_only == 0 { "PASS" } else { "residue printed below" }
    );
    println!(
        "  invariant 3 byte-exact TUs  {agree}/{be_total} reconcile exactly  ({})",
        if agree == be_total { "PASS" } else { "FAIL" }
    );
    let err_terms = cod_only + obj_only + dupes;
    println!(
        "  ERROR TERM on the emitted census: {err_terms} of {comdats} emitted \
         functions = {:.4} pp",
        pct(err_terms, comdats)
    );
    println!(
        "  this scan's emitted census: {in_class}/{emitted} in class ({:.2}%), \
         {unbound} unbound residue",
        pct(in_class, emitted)
    );
    let residue = report.residue_classes();
    if !residue.is_empty() {
        println!("  residue by mangling class:");
        for (k, n) in residue.iter().take(15) {
            println!("    {n:8}  {k}");
        }
    }

    // ---- #134 -------------------------------------------------------------
    let (bs, bt, ics, ict) = report.stall_totals();
    let blhs: usize = report.tus.iter().map(|t| t.blocked_lhs).sum();
    let ilhs: usize = report.tus.iter().map(|t| t.in_class_lhs).sum();
    println!("\n#134  /QXSTALLS SCHEDULING DEMAND (emitted-function units)");
    println!(
        "  BLOCKED  emitted: {bs}/{bt} carry a stall annotation  ({:.2}%)   \
         load-hit-store: {blhs} ({:.2}%)",
        pct(bs, bt),
        pct(blhs, bt)
    );
    println!(
        "  IN-CLASS emitted: {ics}/{ict} carry a stall annotation  ({:.2}%)   \
         load-hit-store: {ilhs} ({:.2}%)   <- THE CONTROL",
        pct(ics, ict),
        pct(ilhs, ict)
    );
    println!(
        "  discrimination: blocked − in-class = {:+.2} pp",
        pct(bs, bt) - pct(ics, ict)
    );
    // The size confound, measured. A blocked function is typically far longer
    // than an in-class one, and a longer body has more chances to stall — so the
    // headline gap has to survive being read inside a size bucket or it is a
    // statement about length.
    println!("  size-stratified (the confound: blocked bodies are longer):");
    println!(
        "    bucket        blocked stalled/total          in-class stalled/total                blocked LHS   in-class LHS"
    );
    for (b, bstall, btot, istall, itot, blhs_b, ilhs_b) in report.size_strata() {
        println!(
            "    {b:<12}  {bstall:>7}/{btot:<7} ({:>6.2}%)   {istall:>7}/{itot:<7} ({:>6.2}%)                {blhs_b:>6} ({:>5.2}%)  {ilhs_b:>6} ({:>5.2}%)",
            pct(bstall, btot),
            pct(istall, itot),
            pct(blhs_b, btot),
            pct(ilhs_b, itot),
        );
    }
    if !qxstalls {
        println!(
            "  (run WITHOUT --qxstalls: both rows must read 0 — that is the \
             negative control for the annotation reader)"
        );
    }
    ExitCode::SUCCESS
}

fn cmd_corpus(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "gen" => cmd_corpus_gen(rest),
        "sample" => cmd_corpus_sample(rest),
        "stats" => cmd_corpus_stats(rest),
        _ => {
            eprintln!("usage: c2rs corpus <gen|sample|stats> [opts]");
            ExitCode::from(2)
        }
    }
}

// `fn opt(rest, key)` used to live here. It was `iter().position(|a| a == key)`
// — the SAME scan boards #194/#195 are about, wearing a helper's name, and nine
// handlers used it. Deleted rather than fixed: a scan cannot refuse what it does
// not look for, so there is no repair short of the parser above. Its doc comment
// also claimed it handled "a leading positional", which it never did; each
// caller re-derived positionals ad hoc, and two of them (`corpus sample`,
// `corpus stats`) ate a flag as the directory name.

static CORPUS_GEN_SPEC: Spec = Spec::new(
    "corpus gen",
    &[
        ("--seed", Arity::Value),
        ("--count", Arity::Value),
        ("--timeout", Arity::Value),
        ("--out", Arity::Value),
    ],
)
.positionals(0);

fn cmd_corpus_gen(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_GEN_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let mut cfg = CorpusConfig::default();
    // `parse().unwrap_or(cfg.seed)` made `--seed abc` run at the DEFAULT seed
    // while the operator believed a seed had been set — a reproducibility claim
    // resting on a value the CLI discarded.
    match args.num("--seed") {
        Ok(Some(v)) => cfg.seed = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num("--count") {
        Ok(Some(v)) => cfg.count = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<u64>("--timeout") {
        Ok(Some(v)) => cfg.timeout = Duration::from_secs(v),
        Ok(None) => {}
        Err(c) => return c,
    }
    let out = args.path("--out").unwrap_or_else(|| PathBuf::from("corpus"));

    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    println!(
        "corpus gen: seed={} count={} timeout={}s -> {}",
        cfg.seed,
        cfg.count,
        cfg.timeout.as_secs(),
        out.display()
    );
    match corpus::generate(&out, &tc, &cfg) {
        Ok(s) => {
            println!(
                "  {} ok, {} codec_fail, {} timeout, {} error ({} distinct sources, {} rows)",
                s.ok,
                s.codec_fail,
                s.timeout,
                s.error,
                s.distinct_sources,
                s.total()
            );
            if s.ok == 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("corpus gen failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// One optional output directory, no options. `rest.first()` was taken verbatim,
/// so `c2rs corpus sample --out /tmp/x` wrote the sample into a directory
/// literally named `--out`.
static CORPUS_SAMPLE_SPEC: Spec = Spec::new("corpus sample", &[]);

fn cmd_corpus_sample(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_SAMPLE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let out = args
        .path_positional()
        .unwrap_or_else(|| PathBuf::from("crates/c2-harness/tests/corpus_sample"));
    match corpus::write_synthetic_sample(&out) {
        Ok(s) => {
            println!("wrote synthetic sample: {} triples -> {}", s.ok, out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("corpus sample failed: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// P1.3 retrieval baseline
// ---------------------------------------------------------------------------

fn cmd_retrieve(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "eval" => cmd_retrieve_eval(rest),
        "index" => cmd_retrieve_index(rest),
        _ => {
            eprintln!("usage: c2rs retrieve <eval|index> <corpus-dir> [opts]");
            ExitCode::from(2)
        }
    }
}

/// `--k 1,5,10`. A bad element used to vanish through `filter_map(parse().ok())`
/// and, if every element was bad, the whole option fell back to the default —
/// so `--k 1,x,10` silently evaluated recall at two cutoffs, not three.
fn parse_ks(args: &Args) -> Result<Vec<usize>, ExitCode> {
    let Some(s) = args.get("--k") else {
        return Ok(vec![1, 5, 10]);
    };
    let mut v = Vec::new();
    for tok in s.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        match tok.parse::<usize>() {
            Ok(k) => v.push(k),
            Err(_) => {
                eprintln!("{}: --k expects integers, got {tok:?}", args.cmd());
                return Err(ExitCode::from(2));
            }
        }
    }
    if v.is_empty() {
        eprintln!("{}: --k names no cutoffs", args.cmd());
        return Err(ExitCode::from(2));
    }
    Ok(v)
}

static RETRIEVE_INDEX_SPEC: Spec = Spec::new("retrieve index", &[]);

fn cmd_retrieve_index(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&RETRIEVE_INDEX_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!("usage: c2rs retrieve index <corpus-dir>");
        return ExitCode::from(2);
    };
    let items = match retrieval::load_items(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("could not load corpus: {e}");
            return ExitCode::FAILURE;
        }
    };
    if items.is_empty() {
        eprintln!("no `ok` triples under {}", dir.display());
        return ExitCode::FAILURE;
    }
    // Behavioral (.text), strict full-obj, and source collision structure.
    let text_classes = class_sizes(items.iter().map(|i| i.text_key.clone()));
    let full_classes = class_sizes(items.iter().map(|i| i.full_key.clone()));
    let src_classes = class_sizes(items.iter().map(|i| i.src_key.clone()));
    let n = items.len();
    let in_multi = |m: &std::collections::BTreeMap<String, usize>| {
        m.values().filter(|&&c| c > 1).sum::<usize>()
    };
    println!("retrieval index: {} ok triples from {}", n, dir.display());
    println!(
        "  distinct sources     : {:>5}   ({} rows in a shared-source class)",
        src_classes.len(),
        in_multi(&src_classes)
    );
    println!(
        "  distinct .text (code): {:>5}   ({} rows / {:.1}% in a code-collision class \u{2265}2)",
        text_classes.len(),
        in_multi(&text_classes),
        in_multi(&text_classes) as f64 / n as f64 * 100.0
    );
    println!(
        "  distinct obj_sha_norm: {:>5}   ({} rows in a full-obj class \u{2265}2; path-polluted)",
        full_classes.len(),
        in_multi(&full_classes)
    );
    let biggest = text_classes.values().copied().max().unwrap_or(0);
    println!("  largest code class   : {biggest} rows");
    println!(
        "  feature              : 256-bin L1-normalized .text byte histogram, cosine NN"
    );
    ExitCode::SUCCESS
}

fn class_sizes<I: Iterator<Item = String>>(keys: I) -> std::collections::BTreeMap<String, usize> {
    let mut m = std::collections::BTreeMap::new();
    for k in keys {
        *m.entry(k).or_insert(0) += 1;
    }
    m
}

static RETRIEVE_EVAL_SPEC: Spec = Spec::new(
    "retrieve eval",
    &[
        ("--k", Arity::Value),
        ("--split", Arity::Value),
        ("--query-div", Arity::Value),
    ],
);

fn cmd_retrieve_eval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&RETRIEVE_EVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!("usage: c2rs retrieve eval <corpus-dir> [--split held-out|loo] [--query-div N] [--k 1,5,10]");
        return ExitCode::from(2);
    };
    let ks = match parse_ks(&args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    // `_ =>` used to swallow every unrecognised spelling, so `--split heldout`
    // ran leave-one-out's OPPOSITE and reported "held-out" as if asked.
    let split = match args.one_of("--split", &["held-out", "loo"]) {
        Ok(v) => v.unwrap_or("held-out"),
        Err(c) => return c,
    };
    if split == "loo" && args.get("--query-div").is_some() {
        eprintln!("retrieve eval: --query-div has no effect under --split loo; refusing rather than dropping it silently");
        return ExitCode::from(2);
    }
    let query_div: u64 = match args.num("--query-div") {
        Ok(v) => v.unwrap_or(5),
        Err(c) => return c,
    };

    let items = match retrieval::load_items(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("could not load corpus: {e}");
            return ExitCode::FAILURE;
        }
    };
    if items.is_empty() {
        eprintln!("no `ok` triples under {}", dir.display());
        return ExitCode::FAILURE;
    }

    let (report, mode) = match split {
        "loo" => (retrieval::evaluate(&items, &items, &ks, true), "leave-one-out".to_string()),
        _ => {
            let (q, idx) = retrieval::split_held_out(items, query_div);
            let m = format!("held-out (query = 1/{query_div} by sha256(id))");
            (retrieval::evaluate(&q, &idx, &ks, false), m)
        }
    };

    println!("P1.3 retrieval baseline — {}", dir.display());
    println!("  split : {mode}");
    println!("  query : {}   index: {}", report.n_query, report.n_index);
    println!(
        "  answerable queries: obj-text(.text)={}  obj-full(sha_norm)={}  exact-source={}",
        report.answerable_text, report.answerable_full, report.answerable_exact
    );
    println!("  recall@k (fraction of queries with a correct IL in top-k):");
    println!("    k    obj-equiv(.text)   obj-full(strict)   exact-source     random(.text)");
    for row in &report.rows {
        println!(
            "    {:<4} {:>10.2}%      {:>10.2}%      {:>10.2}%     {:>10.4}%",
            row.k,
            row.obj_text * 100.0,
            row.obj_full * 100.0,
            row.exact * 100.0,
            row.random_text * 100.0,
        );
    }
    ExitCode::SUCCESS
}

// ---------------------------------------------------------------------------
// T-A IL-space search prototype
// ---------------------------------------------------------------------------

use c2_harness::search::{self, Budget, MoveSet, Perturb};

/// The default solvable-instance roster: straight-line int fixtures that carry
/// literals and/or arithmetic terms (the move set's sites). C++-only; each is
/// captured fresh (no committed IL/obj).
const SEARCH_FIXTURES: &[&str] = &[
    "mvp_edit_addk.cpp",
    "mvp_lit.cpp",
    "mvp_wide.cpp",
    "mvp_add3.cpp",
    "mvp_sub.cpp",
    "mvp_two.cpp",
];

fn cmd_search(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "solve" => cmd_search_solve(rest),
        "eval" => cmd_search_eval(rest),
        "from-retrieval" => cmd_search_from_retrieval(rest),
        "from-lifter" => cmd_search_from_lifter(rest),
        _ => {
            eprintln!("usage: c2rs search <solve <cpp>|eval|from-retrieval <corpus-dir>|from-lifter <corpus-dir> --gens <jsonl>> [--moves full|length] [--steps N] [--compiles N] [--beam K] [--timeout SECS]  (--d 1|2|3 is `search eval` only)");
            ExitCode::from(2)
        }
    }
}

/// The move set. The `_ =>` arm used to map every unrecognised spelling to the
/// full set — and `search eval` then ECHOED the raw string, so `--moves lenght`
/// printed `moves=lenght` in its header while running the full moveset. A report
/// that names a configuration it did not run is worse than one that names none.
/// `one_of` is checked at parse time, so the echo and the behaviour cannot part.
fn search_moveset(args: &Args) -> Result<MoveSet, ExitCode> {
    let mut m = match args.one_of("--moves", &["full", "length"])? {
        Some("length") => MoveSet::length_only(),
        _ => MoveSet::default(),
    };
    // On the real obj-judged path, widen/narrow is obj-INVISIBLE (P0.6a A: c2
    // re-optimizes a re-widthed literal to byte-identical code), so it can never
    // reach a new obj — it only floods the beam with gradient-tied duplicate
    // models that crowd out productive (structure-changing) moves. Drop it from
    // the search moveset unless explicitly re-enabled. (The mock-scorer unit tests
    // keep it via `MoveSet::default()`, where it IS `.ex`-visible.)
    if !args.has("--keep-widen") {
        m.widen_narrow = false;
    }
    Ok(m)
}

fn search_budget(args: &Args) -> Result<Budget, ExitCode> {
    let mut b = Budget::default();
    if let Some(v) = args.num("--steps")? {
        b.max_steps = v;
    }
    if let Some(v) = args.num("--compiles")? {
        b.max_compiles = v;
    }
    if let Some(v) = args.num("--beam")? {
        b.beam_width = v;
    }
    Ok(b)
}

fn search_perturbs(args: &Args) -> Result<Vec<(Perturb, usize)>, ExitCode> {
    // The obj-changing families at d=1, plus AddTerm at d=2 (a gradient-guided
    // two-move recovery) when --d 2 is requested. WidenLit is obj-invisible on
    // the real path (P0.6a A), so it is not in the roster.
    let d: usize = args.num("--d")?.unwrap_or(1);
    let mut v = vec![
        (Perturb::AddTerm, 1),
        (Perturb::LitNudge, 1),
        (Perturb::DropTerm, 1),
    ];
    if d >= 2 {
        v.push((Perturb::AddTerm, 2));
    }
    if d >= 3 {
        v.push((Perturb::AddTerm, 3));
    }
    Ok(v)
}

fn search_timeout(args: &Args) -> Result<Duration, ExitCode> {
    Ok(Duration::from_secs(args.num("--timeout")?.unwrap_or(60)))
}

/// The options every `search` subcommand shares.
const SEARCH_COMMON: &[(&str, Arity)] = &[
    ("--moves", Arity::Value),
    ("--keep-widen", Arity::Flag),
    ("--steps", Arity::Value),
    ("--compiles", Arity::Value),
    ("--beam", Arity::Value),
    ("--timeout", Arity::Value),
];

/// `SEARCH_COMMON` plus the subcommand's own, as one `const` list.
const fn search_spec(cmd: &'static str, opts: &'static [(&'static str, Arity)]) -> Spec {
    Spec { cmd, opts, requires: &[], max_positionals: 1 }
}

/// **`--d` is not here on purpose.** The top-level `search` usage advertised it
/// for every subcommand, but `solve` hardcodes `Perturb::AddTerm, 1` and never
/// calls `search_perturbs`, so `search solve <cpp> --d 3` accepted the option and
/// ran d=1. Refusing it is the honest reading of "this subcommand does not take
/// it"; the ladder lives on `search eval`.
static SEARCH_SOLVE_SPEC: Spec = search_spec("search solve", SEARCH_COMMON);

fn cmd_search_solve(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_SOLVE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = args.path_positional() else {
        eprintln!("usage: c2rs search solve <cpp> [--moves full|length]");
        return ExitCode::from(2);
    };
    let (moves, budget, timeout) = match (|| {
        Ok((search_moveset(&args)?, search_budget(&args)?, search_timeout(&args)?))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }
    let w = scratch("search-solve");
    // An inserted redundant term is the cleanest obj-changing single demo.
    let r = search::solve_instance(&tc, &cpp, Perturb::AddTerm, 1, &moves, &budget, &w, timeout);
    let code = match (&r.outcome, &r.error) {
        (Some(o), _) => {
            println!(
                "{} [{}] -> solved={} steps={} compiles={} best_fuzzy={:.4} ({:?})",
                r.fixture,
                r.perturb.label(),
                o.solved,
                o.steps,
                o.compiles,
                o.best_fuzzy,
                o.reason
            );
            if !o.path.is_empty() {
                println!("  path: {}", o.path.join(" -> "));
            }
            if o.solved {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        (None, Some(e)) => {
            eprintln!("{} -> instance error: {e}", r.fixture);
            ExitCode::FAILURE
        }
        (None, None) => {
            println!("{} -> no perturbation site (skipped)", r.fixture);
            ExitCode::SUCCESS
        }
    };
    let _ = std::fs::remove_dir_all(&w);
    code
}

static SEARCH_EVAL_SPEC: Spec = Spec {
    cmd: "search eval",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--d", Arity::Value),
    ],
    requires: &[],
    max_positionals: 0,
};

fn cmd_search_eval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_EVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let (moves, budget, perturbs, timeout) = match (|| {
        Ok((
            search_moveset(&args)?,
            search_budget(&args)?,
            search_perturbs(&args)?,
            search_timeout(&args)?,
        ))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }
    let fixtures: Vec<PathBuf> = SEARCH_FIXTURES
        .iter()
        .map(|n| c2_harness::fixtures_dir().join(n))
        .collect();

    let w = scratch("search-eval");
    println!(
        "T-A IL-space solve-rate: {} fixtures x {} perturbation families, moves={}, budget steps={} compiles={}",
        fixtures.len(),
        perturbs.len(),
        args.get("--moves").unwrap_or("full"),
        budget.max_steps,
        budget.max_compiles,
    );
    let report = search::solve_rate(&tc, &fixtures, &perturbs, &moves, &budget, &w, timeout);
    for r in &report.instances {
        let tag = format!("{} d{}", r.perturb.label(), r.d);
        match (&r.outcome, &r.error) {
            (Some(o), _) => println!(
                "  {:<20} {:<13} solved={} steps={} compiles={:>3} fuzzy={:.4} {:?}",
                r.fixture, tag, o.solved, o.steps, o.compiles, o.best_fuzzy, o.reason
            ),
            (None, Some(e)) => {
                println!("  {:<20} {:<13} ERROR {}", r.fixture, tag, first_line(e))
            }
            (None, None) => println!("  {:<20} {:<13} skipped (no site)", r.fixture, tag),
        }
    }
    println!("\nsolve-rate by family (attempted excludes no-site/errored):");
    for ((kind, d), (attempted, solved, mean)) in report.by_family() {
        let pct = if attempted > 0 {
            solved as f64 / attempted as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "  {:<10} d{}  {}/{} = {:>5.1}%   mean compiles-to-solve: {:.1}",
            kind.label(),
            d,
            solved,
            attempted,
            pct,
            mean
        );
    }
    let (attempted, solved, mean) = report.tally();
    let pct = if attempted > 0 {
        solved as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "  {:<14} {}/{} = {:>5.1}%   mean compiles-to-solve: {:.1}",
        "OVERALL", solved, attempted, pct, mean
    );
    let _ = std::fs::remove_dir_all(&w);
    if attempted > 0 && solved == attempted {
        ExitCode::SUCCESS
    } else if solved > 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

static SEARCH_FROM_RETRIEVAL_SPEC: Spec = Spec {
    cmd: "search from-retrieval",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--sample", Arity::Value),
        ("--multi", Arity::Value),
        ("--select-seed", Arity::Value),
    ],
    requires: &[],
    max_positionals: 1,
};

fn cmd_search_from_retrieval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_FROM_RETRIEVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!(
            "usage: c2rs search from-retrieval <corpus-dir> [--sample N] [--multi N] [--select-seed N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    // Real-path moveset: drop the obj-invisible widen/narrow (P0.6a A) unless
    // explicitly kept — same rule as `search eval`.
    let (moves, cfg) = match (|| {
        let moves = search_moveset(&args)?;
        let mut cfg = search::FromSeedConfig::default();
        if let Some(v) = args.num("--sample")? {
            cfg.sample = v;
        }
        if let Some(v) = args.num("--multi")? {
            cfg.multi = v;
        }
        if let Some(v) = args.num("--select-seed")? {
            cfg.select_seed = v;
        }
        if let Some(v) = args.num("--steps")? {
            cfg.budget.max_steps = v;
        }
        if let Some(v) = args.num("--compiles")? {
            cfg.budget.max_compiles = v;
        }
        if let Some(v) = args.num("--beam")? {
            cfg.budget.beam_width = v;
        }
        if let Some(v) = args.num::<u64>("--timeout")? {
            cfg.timeout = Duration::from_secs(v);
        }
        Ok((moves, cfg))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }

    let w = scratch("search-from-retrieval");
    println!(
        "T-A from-unrelated-seed: sample={} (multi={}) select-seed={} budget steps={} compiles={} beam={} timeout={}s",
        cfg.sample,
        cfg.multi,
        cfg.select_seed,
        cfg.budget.max_steps,
        cfg.budget.max_compiles,
        cfg.budget.beam_width,
        cfg.timeout.as_secs(),
    );
    let report = match search::from_retrieval_eval(&tc, &dir, &moves, &cfg, &w) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("from-retrieval eval failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    let _ = std::fs::remove_dir_all(&w);

    println!("  corpus items: {}", report.n_items);
    println!("  per-target (target[fns] <- seed[fns] : class):");
    for r in &report.records {
        let seed = match (&r.seed_id, r.seed_fns) {
            (Some(id), Some(f)) => format!("{id}[{f}fns]"),
            (Some(id), None) => format!("{id}[?]"),
            _ => "<none>".to_string(),
        };
        let mut line = format!(
            "    {}[{}fns] <- {:<14} {:<17}",
            r.target_id,
            r.target_fns,
            seed,
            r.class.label()
        );
        if let Some(o) = &r.outcome {
            line.push_str(&format!(
                " solved={} steps={} compiles={} fuzzy={:.4} {:?}",
                o.solved, o.steps, o.compiles, o.best_fuzzy, o.reason
            ));
        } else if !r.detail.is_empty() {
            line.push_str(&format!(" ({})", r.detail));
        }
        println!("{line}");
        if let Some(w2) = &r.outcome_wholetext {
            let p = r.outcome.as_ref().map(|o| o.solved).unwrap_or(false);
            println!(
                "        per-fn vs whole-.text: per-fn solved={} fuzzy={:.4} | whole solved={} fuzzy={:.4}",
                p,
                r.outcome.as_ref().map(|o| o.best_fuzzy).unwrap_or(0.0),
                w2.solved,
                w2.best_fuzzy,
            );
        }
    }

    println!("\n  failure taxonomy:");
    for (class, n) in report.class_counts() {
        println!("    {:<17} {}", class.label(), n);
    }
    let (searched, solved) = report.search_tally();
    let pct = if searched > 0 {
        solved as f64 / searched as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "\n  HEADLINE: from-unrelated-seed solve-rate = {solved}/{searched} = {pct:.1}% (searched = in-scope, non-trivial)"
    );
    ExitCode::SUCCESS
}

static SEARCH_FROM_LIFTER_SPEC: Spec = Spec {
    cmd: "search from-lifter",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--gens", Arity::Value),
        ("--k", Arity::Value),
        ("--limit", Arity::Value),
    ],
    requires: &[],
    max_positionals: 1,
};

fn cmd_search_from_lifter(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_FROM_LIFTER_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!(
            "usage: c2rs search from-lifter <corpus-dir> --gens <jsonl> [--k K] [--limit N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    let Some(gens_path) = args.path("--gens") else {
        eprintln!("from-lifter: --gens <jsonl> required (rows {{\"id\",\"generations\":[...]}})");
        return ExitCode::from(2);
    };
    let (k, limit, moves, budget, timeout) = match (|| {
        let k: usize = args.num("--k")?.unwrap_or(5);
        let limit: usize = args.num("--limit")?.unwrap_or(0);
        let moves = search_moveset(&args)?;
        let mut budget = search_budget(&args)?;
        // Bounded per-generation defaults (many generations x targets share one
        // CPU). The guard is PRESENCE, and that used to matter: with
        // `.parse().ok()` swallowing a bad value, `--compiles abc` left
        // `Budget::default()`'s 400 instead of this bounded 200 — a typo doubled
        // the compile budget in silence. `num` refuses first, so presence and
        // parse-success now coincide.
        if args.get("--steps").is_none() {
            budget.max_steps = 8;
        }
        if args.get("--compiles").is_none() {
            budget.max_compiles = 200;
        }
        if args.get("--beam").is_none() {
            budget.beam_width = 4;
        }
        let timeout = Duration::from_secs(args.num::<u64>("--timeout")?.unwrap_or(25));
        Ok((k, limit, moves, budget, timeout))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }

    let gens = match search::load_lifter_gens(&gens_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("from-lifter: reading {}: {e}", gens_path.display());
            return ExitCode::FAILURE;
        }
    };
    println!(
        "T-B lifter byte-exact eval: gens={} targets, k={}, limit={}, budget steps={} compiles={} beam={} timeout={}s",
        gens.len(),
        k,
        limit,
        budget.max_steps,
        budget.max_compiles,
        budget.beam_width,
        timeout.as_secs(),
    );

    let w = scratch("search-from-lifter");
    let report =
        match search::from_lifter_eval(&tc, &dir, &gens, k, limit, &moves, &budget, timeout, &w) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("from-lifter eval failed: {e}");
                let _ = std::fs::remove_dir_all(&w);
                return ExitCode::FAILURE;
            }
        };
    let _ = std::fs::remove_dir_all(&w);

    println!("  corpus ok-rows: {}", report.n_items);
    println!("  per-target (id[fns] class : detail):");
    for r in &report.records {
        let slot = match r.solved_slot {
            Some(s) => format!("slot{s}"),
            None => "-".to_string(),
        };
        println!(
            "    {:<9}[{}fns] cap={}/{} {:<10} {:<6} {}",
            r.target_id, r.target_fns, r.captured, r.k, r.class.label(), slot, r.detail
        );
    }

    let (attempted, pass1, passk) = report.tally();
    let no_compile = report
        .records
        .iter()
        .filter(|r| r.class == search::LifterClass::NoCompile)
        .count();
    let errors = report
        .records
        .iter()
        .filter(|r| r.class == search::LifterClass::Error)
        .count();
    let p1 = if attempted > 0 {
        pass1 as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    let pk = if attempted > 0 {
        passk as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "\n  attempted(searched)={attempted} no-compile-targets={no_compile} target-errors={errors}"
    );
    println!("  P1.3 retrieval control floor: 9.6% pass@1");
    println!(
        "  HEADLINE: lifter byte-exact pass@1 = {pass1}/{attempted} = {p1:.1}%   pass@{k} = {passk}/{attempted} = {pk:.1}%"
    );
    ExitCode::SUCCESS
}

static CORPUS_STATS_SPEC: Spec = Spec::new("corpus stats", &[]);

fn cmd_corpus_stats(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_STATS_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!("usage: c2rs corpus stats <dir>");
        return ExitCode::from(2);
    };
    match corpus::load_manifest(&dir) {
        Ok(rows) => {
            let ok = rows.iter().filter(|r| r.status == "ok").count();
            let rt = rows
                .iter()
                .filter(|r| r.codec_roundtrip == Some(true))
                .count();
            let toks: usize = rows.iter().filter_map(|r| r.ex_token_count).sum::<i64>() as usize;
            println!(
                "{}: {} rows, {} ok, {} codec round-trip, {} total .ex tokens",
                dir.display(),
                rows.len(),
                ok,
                rt,
                toks
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("could not read manifest: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// gap — real-workload gap scan
// ---------------------------------------------------------------------------

use c2_harness::gap::{gap_scan, GapConfig, TuClass};

/// `c2rs gap --list FILE --flags-file FILE [--cwd DIR] …` — scan real TUs,
/// classify each (capture-fail / vocab-gap / codegen-gap / mismatch / match),
/// and rank the blockers. Exit is non-zero only on a *correctness* signal
/// (`mismatch` TUs or a replay-soundness divergence) or a harness error —
/// gaps themselves are the expected measurement, not a failure.
static GAP_SPEC: Spec = Spec {
    cmd: "gap",
    opts: &[
        ("--list", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--limit", Arity::Value),
        ("--jobs", Arity::Value),
        ("--replay-every", Arity::Value),
        ("--jsonl", Arity::Value),
        ("--work", Arity::Value),
        ("--cache", Arity::Value),
        ("--no-cache", Arity::Flag),
        ("--validate-cache", Arity::Value),
    ],
    requires: &[],
    max_positionals: 0,
};

fn cmd_gap(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&GAP_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let (list_file, flags_file) = (args.path("--list"), args.path("--flags-file"));
    let cwd = args.path("--cwd");
    let jsonl = args.path("--jsonl");
    let work = args.path("--work");
    // `--limit`/`--jobs`/`--replay-every`/`--validate-cache` used to exit 2 with
    // NO message when the value did not parse. `num` names the option and echoes
    // the value it choked on.
    let limit: Option<usize> = match args.num("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let jobs: usize = match args.num("--jobs") {
        Ok(Some(v)) => v,
        Ok(None) => std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        Err(c) => return c,
    };
    let replay_every: usize = match args.num("--replay-every") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    let validate_cache: usize = match args.num("--validate-cache") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    // Capture cache: ON by default (roadmap #15). The key is content-addressed
    // (source bytes + flags + toolchain + workload-tree identity), never mtimes;
    // `--no-cache` bypasses it and `--validate-cache N` re-captures every Nth
    // hit and byte-compares. Default root is under the gitignored `work/`.
    //
    // `--cache` and `--no-cache` used to be ORDER-DEPENDENT, because each simply
    // assigned as the scan walked: `--cache X --no-cache` disabled the cache and
    // dropped X, while `--no-cache --cache X` used X. One of those two spellings
    // did something other than what it says, and which one depended on argument
    // order. They are contradictory, so refuse both spellings.
    if args.has("--no-cache") && args.get("--cache").is_some() {
        eprintln!("gap: --cache and --no-cache contradict each other; give one");
        return ExitCode::from(2);
    }
    // `--validate-cache N` re-captures every Nth *hit*, so with no cache there
    // are no hits and nothing is validated — but the report still printed
    // "validating every Nth hit" under a `DISABLED (--no-cache)` cache line.
    if args.has("--no-cache") && validate_cache > 0 {
        eprintln!(
            "gap: --validate-cache has nothing to validate with --no-cache; refusing rather \
             than reporting a validation that cannot run"
        );
        return ExitCode::from(2);
    }
    let cache: Option<PathBuf> = if args.has("--no-cache") {
        None
    } else {
        Some(args.path("--cache").unwrap_or_else(|| {
            std::env::var_os("C2RS_GAP_CACHE")
                .map(PathBuf::from)
                // `main_repo_root`, not `repo_root`: the latter is
                // CARGO_MANIFEST_DIR and so resolves to the *worktree* a lane's
                // binary was built in, which is how 50 separate caches came to
                // exist. See its doc comment for why this is resolved in code
                // rather than exported as an env var.
                .unwrap_or_else(|| {
                    c2_harness::provenance::main_repo_root().join("work/capture-cache")
                })
        }))
    };
    let (Some(list_file), Some(flags_file)) = (list_file, flags_file) else {
        eprintln!(
            "usage: c2rs gap --list FILE --flags-file FILE [--cwd DIR] [--limit N] \
             [--jobs N] [--replay-every N] [--jsonl PATH] [--work DIR]\n\
             (generate the dc3 workload inputs with scripts/gen_dc3_workload.sh)"
        );
        return ExitCode::from(2);
    };

    let read_tokens = |p: &PathBuf, split: bool| -> std::io::Result<Vec<String>> {
        let text = std::fs::read_to_string(p)?;
        let mut out = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if split {
                out.extend(line.split_whitespace().map(String::from));
            } else {
                out.push(line.to_string());
            }
        }
        Ok(out)
    };
    let sources = match read_tokens(&list_file, false) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --list {}: {e}", list_file.display());
            return ExitCode::FAILURE;
        }
    };
    let flags = match read_tokens(&flags_file, true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --flags-file {}: {e}", flags_file.display());
            return ExitCode::FAILURE;
        }
    };

    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep IL bundles during capture)");
        return ExitCode::SUCCESS;
    }
    if replay_every > 0 && !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed for --replay-every)");
        return ExitCode::SUCCESS;
    }

    let cfg = GapConfig {
        sources,
        flags,
        cwd,
        limit,
        jobs,
        replay_every,
        jsonl,
        work: work.unwrap_or_else(|| scratch("gap")),
        cache,
        validate_cache,
    };
    let total = cfg.limit.unwrap_or(cfg.sources.len()).min(cfg.sources.len());
    println!(
        "gap scan: {total} TUs, {} flags, jobs={}, replay-every={}",
        cfg.flags.len(),
        cfg.jobs,
        cfg.replay_every
    );
    println!(
        "  capture cache: {}{}",
        match &cfg.cache {
            Some(p) => p.display().to_string(),
            None => "DISABLED (--no-cache)".to_string(),
        },
        if cfg.validate_cache > 0 {
            format!("  (validating every {}th hit)", cfg.validate_cache)
        } else {
            String::new()
        }
    );
    // Roadmap #46/#48: name the corpus, the binary, and the loader BEFORE the
    // numbers. A moved corpus once matched on `fn_total` and a stale wibo once
    // faked a replay alarm; neither was visible in any line of the old report.
    print!("{}", Provenance::collect(&tc, cfg.cwd.as_deref()).render());

    let t0 = std::time::Instant::now();
    let report = match gap_scan(&tc, &cfg, &|n, tot, r| {
        println!(
            "  [{n}/{tot}] {:<12} {}{}",
            r.class.label(),
            r.src,
            if r.reason.is_empty() || r.class == TuClass::Match {
                String::new()
            } else {
                format!("  ({})", r.reason)
            }
        );
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gap scan failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let elapsed = t0.elapsed();

    let n = report.results.len().max(1);
    println!("\nGAP REPORT ({} TUs in {:.1}s)", report.results.len(), elapsed.as_secs_f64());
    for class in [
        TuClass::Match,
        TuClass::Mismatch,
        TuClass::CodegenGap,
        TuClass::VocabGap,
        TuClass::PortError,
        TuClass::CaptureFail,
    ] {
        let c = report.count(class);
        println!("  {:<13} {:>5}  {:>5.1}%", class.label(), c, 100.0 * c as f64 / n as f64);
    }
    let (checked, diverged) = report.replay_stats();
    if checked > 0 {
        println!("  replay soundness: {checked} checked, {diverged} diverged");
    }
    let cs = &report.cache;
    if cfg.cache.is_some() {
        println!(
            "  capture cache: {} hit, {} miss, {} uncacheable  |  validator: {} re-captured \
             and agreed ({} of them only after zeroing the COFF TimeDateStamp), {} POISONED",
            cs.hits, cs.misses, cs.bypassed, cs.validated, cs.timestamp_only, cs.poisoned
        );
        for line in cs.poison_detail.iter().take(10) {
            println!("    POISONED {line}");
        }
        if cs.poison_detail.len() > 10 {
            println!("    … and {} more", cs.poison_detail.len() - 10);
        }
    }

    // P2b function-level census. The TU ladder above is all-or-nothing, so it
    // reads 0% until a whole TU comes in class; this is the fine-grained
    // numerator that actually moves per widening step, plus the ranked
    // histogram that chooses the next step (docs/ROADMAP.md §G5).
    let (in_class, fn_total) = report.fn_coverage();
    if fn_total > 0 {
        println!(
            "\n  FUNCTION CENSUS (P2b): {in_class}/{fn_total} functions in class ({:.2}%)",
            100.0 * in_class as f64 / fn_total as f64
        );
        // The census/gate cross-check (roadmap #44). The numerator above is the
        // public claim, so its error term is printed beside it on every scan
        // rather than being characterized once and forgotten: these are the
        // functions the census calls in class and `PortC2`'s own per-function
        // selector refuses. It must read 0.
        let disagree = report.fn_gate_disagreement();
        if disagree == 0 {
            println!("  census/gate disagreement: 0  (the port accepts every function above)");
        } else {
            println!(
                "  census/gate DISAGREEMENT: {disagree} ({:.2}% of the numerator) — the census \
                 OVER-CLAIMS by this much",
                100.0 * disagree as f64 / in_class.max(1) as f64
            );
            for (key, count) in report.fn_gate_histogram().iter().take(15) {
                println!("    {count:>7}  {key}");
            }
        }
        // The `.gl` binding invariants (D14). A binding decides *which* symbol a
        // token names, and a green differential cannot grade a correspondence
        // (`docs/GAPS.md` §6). These are the two facts the container settles by
        // itself, and both have a known answer of 0.
        let binds = report.bind_check_histogram();
        if !binds.is_empty() {
            let bad = report.bind_violations();
            if bad == 0 {
                println!(
                    "  .gl binding invariants: 0 violations (every generated destructor \
                     resolves to a destructor)"
                );
            } else {
                println!("  .gl binding VIOLATIONS: {bad} — a token bound to the wrong symbol");
            }
            for (key, count) in binds.iter().take(8) {
                println!("    {count:>7}  {key}");
            }
        }
        // ---- The EMITTED-function census (`docs/GAPS.md` §8) ----------------
        //
        // The headline above counts IL bodies. c2 emits about 7 % of them, and
        // for a body it never emits "in class" is a parser-only claim no byte
        // compare has ever graded or ever can. This block is the same numerator
        // restricted to functions that actually appear in an obj, and it is a
        // FLOOR: every emitted symbol the binding could not claim is printed as
        // residue, never folded into the numerator or out of the denominator.
        let (emit_in_class, emitted) = report.emit_coverage();
        if emitted > 0 {
            println!(
                "\n  EMITTED CENSUS (§8): {emit_in_class}/{emitted} emitted functions in class \
                 ({:.2}%)",
                100.0 * emit_in_class as f64 / emitted as f64
            );
            let (gen, unexplained) = report.emit_residue();
            let bound = report.emit_total("emit-bound");
            println!(
                "    bound {bound}  |  residue {}: {gen} compiler-generated (no IL body), \
                 {unexplained} unexplained  ({:.2}% of the denominator)",
                gen + unexplained,
                100.0 * (gen + unexplained) as f64 / emitted as f64
            );
            println!(
                "    ceiling if every residue symbol were in class: {} ({:.2}%)",
                emit_in_class + gen + unexplained,
                100.0 * (emit_in_class + gen + unexplained) as f64 / emitted as f64
            );
            // Ground truth. On a byte-exact TU the oracle has already graded the
            // whole symbol table, so this binding's answer there is checkable
            // rather than merely self-consistent. Known answer 0.
            let mtu = report.count(TuClass::Match);
            let mres = report.emit_match_tu_residue();
            if mres == 0 {
                println!(
                    "    ground truth: {mtu} byte-exact TUs, every emitted symbol bound to an \
                     in-class row (residue 0)"
                );
            } else {
                println!(
                    "    ground truth VIOLATED: {mres} emitted symbols on the {mtu} byte-exact \
                     TUs did not bind to an in-class row — the BINDING is wrong, not the port"
                );
            }
            let broken = report.emit_total("emit-accounting-broken");
            let unreadable = report.emit_total("emit-obj-unreadable");
            println!(
                "    binding: {} records, {} nameless, {} before the first row, {} row-conflicts, \
                 {} name-conflicts, {broken} accounting breaks, {unreadable} unreadable objs",
                report.emit_total("emit-records"),
                report.emit_total("emit-record-nameless"),
                report.emit_total("emit-record-outside"),
                report.emit_total("emit-row-conflict"),
                report.emit_total("emit-name-conflict"),
            );
            // ARITY, beside totality and never instead of it (#144). The counts
            // answer different questions: `records` is how many records the
            // FRAMING found, `record-offsets` is their contents. A change to the
            // NAME scan must move `nameless` above and leave both of these fixed
            // — which is how W-VGL's `26`-separator repair was held to being a
            // naming repair (records 1,515,160 before and after, nameless
            // 152,941 -> 420).
            println!(
                "    binding arity (#144 — residue 0 is not a control): {} record offsets \
                 against {} records, {} arity breaks (known answer 0)",
                report.emit_total("emit-record-offsets"),
                report.emit_total("emit-records"),
                report.emit_total("emit-arity-broken"),
            );
            // What the unexplained residue IS. A residue reported only as a
            // number cannot be attacked and cannot be checked; these rows say
            // whether it is a population c2 synthesizes (concentrated in the
            // special-member classes) or the binding losing ordinary functions.
            let by_class: Vec<(String, usize)> = report
                .emit_histogram()
                .into_iter()
                .filter(|(k, _)| k.starts_with("emit-residue-unbound|"))
                .collect();
            for (key, n) in by_class.iter().take(10) {
                println!(
                    "      residue {:<20} {n:>7} ({:>5.1}% of the unexplained)",
                    key.trim_start_matches("emit-residue-unbound|"),
                    100.0 * *n as f64 / unexplained.max(1) as f64
                );
            }
            // The payoff metric's leading indicator, as a distribution.
            let buckets = [0usize, 1, 10, 100, 1000];
            let dist: Vec<String> = buckets
                .iter()
                .map(|b| format!("≤{b}: {}", report.near_match_tus(*b).len()))
                .collect();
            println!("    TU distance to matching (blocked functions) — {}", dist.join(", "));
            // The same distribution over the population the goal is written in.
            // Published beside the body one because they are different numbers
            // AND a different order: `Rand2.cpp` is 8 blocked bodies / 2 blocked
            // emitted, `vec.cpp` is 565 blocked bodies / 0 blocked emitted.
            let dist_e: Vec<String> = buckets
                .iter()
                .map(|b| format!("≤{b}: {}", report.near_match_tus_emitted(*b).len()))
                .collect();
            println!(
                "    TU distance to matching (blocked EMITTED functions) — {}",
                dist_e.join(", ")
            );
            // The ceiling neither distance can see: the port emits one `.text`
            // COMDAT per `.ex` function segment and has no emit-set model, so a
            // TU whose segment count differs from its obj's COMDAT-leader count
            // cannot match however good the codegen gets.
            let reach = report.emit_set_reachable_tus();
            let graded = report
                .results
                .iter()
                .filter(|r| r.class != c2_harness::gap::TuClass::CaptureFail)
                .count();
            let viol = report.emit_set_violations();
            println!(
                "    emit-set ceiling: {} of {graded} graded TUs have `.ex` segments == obj `.text` \
                 COMDATs — the most TU match can reach before ROADMAP §8.3 Phase 7 \
                 (violations among matching TUs, must be 0: {viol})",
                reach.len()
            );
            // **W-EMITSET — the ceiling on a MODEL, which is a different and
            // lower number than the ceiling on today's model-free port.**
            //
            // The line above compares two counts. A model has to reproduce the
            // reference `.text` COMDAT *set*, and it can only ever emit a COMDAT
            // for a body this bundle carries, under the name the binding gives
            // it. So the bound is per TU: does every emitted symbol bind (today),
            // and failing that does every emitted symbol at least HAVE a `.gl`
            // body record (repaired)? The gap between the two is `bind.rs` work;
            // what remains after it is a wall that needs COMDAT synthesis.
            let (c_today, c_repaired, c_wall) = (
                report.emit_total("emit-set-ceiling-today"),
                report.emit_total("emit-set-ceiling-repaired"),
                report.emit_total("emit-set-ceiling-wall"),
            );
            let (u_body, u_none) = (
                report.emit_total("emit-unbound-has-record"),
                report.emit_total("emit-unbound-no-record"),
            );
            println!(
                "    emit-set MODEL ceiling: {c_today} of {graded} TUs bind every emitted symbol \
                 today; {c_repaired} would if `bind.rs` lost none; {c_wall} carry an emitted \
                 symbol with NO `.gl` body record and are a wall for any segment-driven model"
            );
            println!(
                "      unbound emitted symbols: {u_body} have a body record (instrument defect), \
                 {u_none} have none (wall) — nesting invariant, must hold: \
                 today <= repaired ({}), repaired + wall == graded ({})",
                c_today <= c_repaired,
                c_repaired + c_wall == graded
            );

            let eh = report.emit_blocker_histogram();
            if !eh.is_empty() {
                let blocked: usize = eh.iter().map(|(_, n)| *n).sum();
                println!("    the widening order OVER EMITTED CODE ({blocked} blocked):");
                for (feature, count) in eh.iter().take(20) {
                    println!(
                        "      {count:>7} ({:>5.1}%)  {feature}",
                        100.0 * *count as f64 / blocked as f64
                    );
                }
                if eh.len() > 20 {
                    println!("      … and {} more distinct features", eh.len() - 20);
                }
            }
            // The payoff metric's leading indicator, with the emitted census
            // beside it: these are the TUs whose remaining distance is small
            // enough to be worked directly.
            let near = report.near_match_tus(100);
            if !near.is_empty() {
                println!(
                    "    TUs within 100 blocked functions of matching: {} \
                     (blocked | emitted in-class/emitted | src)",
                    near.len()
                );
                for r in near.iter().take(60) {
                    let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                    let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                    println!(
                        "      {:>5} | {i:>4}/{e:<4} | {} [{}]",
                        r.fn_total - r.fn_in_class,
                        r.src,
                        r.class.label()
                    );
                }
                if near.len() > 60 {
                    println!("      … and {} more", near.len() - 60);
                }
            }
        }

        let hist = report.fn_blocker_histogram();
        if !hist.is_empty() {
            println!("  blocking features (the widening order):");
            let blocked: usize = hist.iter().map(|(_, n)| *n).sum();
            for (feature, count) in hist.iter().take(20) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {feature}",
                    100.0 * *count as f64 / blocked as f64
                );
            }
            if hist.len() > 20 {
                println!("    … and {} more distinct features", hist.len() - 20);
            }
            // **The two axes do not share a vocabulary, and nothing else says so.**
            // These keys come from `mcall`'s walk, which does not decode `0x64`
            // (the by-value return's materialize) or `0x67` (virtual dispatch); the
            // control-flow axis below does, and spells them by name. So an
            // `op-0x64` row here and a `cf-…` row there are the *same construct
            // under two vocabularies*, and a ranking that adds or compares them is
            // comparing an unnamed byte with a named production. Renaming these
            // from the statement layer would be a census key change with no
            // production behind it — the rung that widens `mcall` renames them,
            // with the 1:1 proof (`docs/IL_DECODE_REACH.md` §11.2).
            if hist.iter().any(|(k, _)| k.contains("op-0x64") || k.contains("op-0x67")) {
                println!(
                    "    NOTE: `op-0x64`/`op-0x67` above spell hex because mcall's walk does \
                     not decode them; the control-flow axis below does. Do not rank across \
                     the two axes on those rows — different vocabularies."
                );
            }
        }
        // The D6 frame axis (`docs/IL_CALL_IN_EXPR.md` §18). The blocking-feature
        // histogram above ranks by *size*; this one says whether a row's lowering
        // can be local at all, and `calls-2plus` is the half that provably cannot
        // — two calls means LR is clobbered, so the body needs a frame.
        let [c0, c1, c2p] = report.frame_class_totals();
        let seen = c0 + c1 + c2p;
        if seen > 0 {
            println!("  frame class (CALL tokens per body — decode-only, §18):");
            for (label, n) in [("calls-0", c0), ("calls-1", c1), ("calls-2plus", c2p)] {
                println!(
                    "    {n:>7} ({:>5.1}%)  {label}",
                    100.0 * n as f64 / seen as f64
                );
            }
            let frames = report.fn_frame_histogram();
            for (key, count) in frames
                .iter()
                .filter(|(k, _)| k.starts_with("calls-2plus|"))
                .take(10)
            {
                println!("    {count:>7}           {key}");
            }
        }
        // The control-flow axis (roadmap #25/#61) — DECODE ONLY, and the sizing of
        // the block-IR restructure. `cflow-*` rows are bodies whose statement layer
        // decoded end to end, so their CFG is known; `cf-*` rows are where the
        // statement-layer decoder itself stopped, which is the residue of the
        // grammar and the next decode rung's own widening order.
        let (dec, undec) = report.cflow_decoded_totals();
        let cf_seen = dec + undec;
        if cf_seen > 0 {
            println!(
                "  control-flow class (statement layer, decode-only): {dec} of {cf_seen} bodies \
                 decoded end to end ({:.1}%)",
                100.0 * dec as f64 / cf_seen as f64
            );
            let cflow = report.fn_cflow_histogram();
            for (key, count) in cflow.iter().filter(|(k, _)| !k.contains('|')).take(14) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {key}",
                    100.0 * *count as f64 / cf_seen as f64
                );
            }
            // …and what each decoded shape is worth **if it were lowered**: the
            // shape crossed with what else the body is blocked on. A row whose
            // census key is an `expr-*` feature is waiting on the expression layer
            // too; the `+expr-modeled` rows are the ones waiting on control flow
            // alone, and their total is the number this restructure is worth today.
            for (key, count) in cflow
                .iter()
                .filter(|(k, _)| k.contains('|') && !k.starts_with("cflow-straight"))
                .take(12)
            {
                println!("    {count:>7}           {key}");
            }
        }
        // The EH axis (`docs/EH_RECORDS.md` §9.4, §10) — DECODE ONLY, and the
        // sizing of the EH phase. `eh-state0` is the cheap side: a destructible
        // object is live but no call crosses it, so `maxState = 0` and no handler
        // prefix, no second `.pdata`, no `.rdata`, no funclet is emitted.
        // `eh-state1` is the whole of §1–§5.
        let ehh = report.fn_eh_histogram();
        let eh_seen: usize = ehh
            .iter()
            .filter(|(k, _)| !k.contains('|'))
            .map(|(_, n)| *n)
            .sum();
        let get = |k: &str| -> usize {
            ehh.iter().find(|(a, _)| a == k).map(|(_, n)| *n).unwrap_or(0)
        };
        if eh_seen > 0 {
            let need = get("eh-state1");
            println!(
                "  EH class (maxState, decode-only): {need} of {eh_seen} bodies have maxState >= 1 \
                 and need the whole EH record ({:.1}%)",
                100.0 * need as f64 / eh_seen as f64
            );
            // Totals over EVERY function, with the blocked subtotal beside each —
            // the two are different populations and a table that prints only one
            // of them invites the reader to rank the other by mistake.
            for (key, count) in ehh.iter().filter(|(k, _)| !k.contains('|')) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<12}  blocked {:>7}",
                    100.0 * *count as f64 / eh_seen as f64,
                    get(&format!("{key}|BLOCKED"))
                );
            }
            // The BLOCKED cross: what a rung on this side would actually have to
            // widen. In-class rows are excluded here by construction — they are
            // printed separately below as the control group they are.
            println!("    blocked residue, by census key (accepted shapes excluded):");
            for (key, count) in ehh
                .iter()
                .filter(|(k, _)| k.contains("|BLOCKED|") && !k.starts_with("eh-none|"))
                .take(16)
            {
                println!("    {count:>7}           {key}");
            }
            // …and the control group, labelled as one. These are functions the
            // port ACCEPTS. They are here because the cheap side of the boundary
            // is exactly where the accepted `empty-dtor-*` shapes live, so one of
            // them reading anything but `eh-state0` indicts the axis.
            println!("    in-class control group (ACCEPTED functions, never a rung):");
            for (key, count) in ehh
                .iter()
                .filter(|(k, _)| k.contains("|INCLASS|") && !k.starts_with("eh-none|"))
                .take(10)
            {
                println!("    {count:>7}           {key}");
            }
            // The migration cross against the refuted statement-count axis.
            println!("    maxState x statement-count (the refuted predicate, for reconciliation):");
            for (key, count) in ehh.iter().filter(|(k, _)| k.starts_with("eh-migrate|")) {
                if key.ends_with("|eh-none") && key.starts_with("eh-migrate|eh-none") {
                    continue;
                }
                println!("    {count:>7}           {key}");
            }
        }
        // The two DISPATCH axes — DECODE ONLY, and the answer to a question no
        // census key can be asked: **which recognizer looked at this body, and
        // where inside it did the refusal happen.**
        //
        // The blocking-feature histogram above names the *construct*. It cannot
        // say whether a widening could reach the body at all: a member call that
        // is a store's right-hand side or a plain call's argument gets the same
        // `expr-call-in-expr-recv-*` key as one that is the whole body, and only
        // the last of the three ever enters a member-call production. Nor can it
        // say whether a row is a missing construct or a **private limit inside a
        // recognizer that already ships** — which has been the answer six rungs
        // running.
        // **THE GRAMMAR-COMPLETENESS AXIS** (roadmap §9.11 / §9.14) — is anything
        // hiding behind a row, or is its count directly a widening estimate?
        //
        // Printed as its own axis, and printed WHOLE, because the fact has two
        // producers that write it into two different halves of the census key
        // (`-whole`/`-more` and `:eof`/`:mid`). WR1 moved 39,967 functions from
        // one encoding to the other; every ranking table built by grepping the
        // key for `-whole` has under-counted that family by 18,931 since. The
        // residue row `complete-none` is printed rather than suppressed for the
        // same reason `prod-entered-untagged` is: an axis whose residue is
        // invisible cannot be audited, and "reported as nothing" and "measured
        // at zero" must never share a rendering.
        let comph = report.fn_complete_histogram();
        let comp_bare: Vec<_> = comph.iter().filter(|(k, _)| !k.contains('|')).collect();
        let comp_seen: usize = comp_bare.iter().map(|(_, n)| *n).sum();
        if comp_seen > 0 {
            println!(
                "  grammar completeness (decode-only): {comp_seen} of {fn_total} bodies read"
            );
            if comp_seen != fn_total {
                println!(
                    "    AXIS UNDER-REPORTS: {} bodies have no completeness reading",
                    fn_total - comp_seen.min(fn_total)
                );
            }
            let whole: usize = comph
                .iter()
                .filter(|(k, _)| k.ends_with("|BLOCKED") && k.starts_with("complete-whole"))
                .map(|(_, n)| *n)
                .sum();
            for (key, count) in &comp_bare {
                let blocked = comph
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<30}  blocked {blocked:>7}",
                    100.0 * *count as f64 / comp_seen as f64
                );
            }
            // The join the roadmap actually ranks by, across BOTH producers —
            // the number §9.13 had to re-derive by hand.
            println!(
                "    grammar-COMPLETE and blocked, both producers summed: {whole}"
            );
        }

        let disph = report.fn_dispatch_histogram();
        let prodh = report.fn_prod_histogram();
        let (disp_seen, prod_seen) = report.dispatch_axis_totals();
        if disp_seen > 0 {
            // Both axes must sum to the census. Stated as an equality rather than
            // left implicit: a short count means bodies took an arm nobody tagged,
            // and the resulting table would under-report every row in it while
            // looking perfectly well formed.
            println!(
                "  body dispatch (which recognizer claimed the body, decode-only): {disp_seen} of \
                 {fn_total} bodies tagged on the dispatch axis, {prod_seen} on the production axis"
            );
            if disp_seen != fn_total || prod_seen != fn_total {
                println!(
                    "    AXIS UNDER-REPORTS: {} / {} bodies are missing a tag — every row below \
                     is a lower bound",
                    fn_total - disp_seen.min(fn_total),
                    fn_total - prod_seen.min(fn_total)
                );
            }
            // Every bare row, untruncated. A dispatch arm that is dropped for
            // being small reads as an arm no body takes, and the whole reason
            // this axis exists is that "reported as nothing" and "measured at
            // zero" had become the same rendering.
            for (key, count) in disph.iter().filter(|(k, _)| !k.contains('|')) {
                let blocked = disph
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<32}  blocked {blocked:>7}",
                    100.0 * *count as f64 / disp_seen as f64
                );
            }
            // The BLOCKED cross for the arms that cannot reach a member-call
            // production at all. **This is the row set that says a member-call
            // widening cannot serve these bodies** — they are the same construct
            // in a statement position the productions never see.
            //
            // Selected by EXCLUDING the two arms that do reach a production
            // (`disp-member-call`, and `disp-assign` which is only ever arrived at
            // by falling through one), rather than by listing the arms that do
            // not: a list of names silently drops any arm added later, and an arm
            // that vanishes from this table is a population reported as zero.
            println!("    blocked residue of the arms that never enter a member-call production:");
            for (key, count) in disph
                .iter()
                .filter(|(k, _)| {
                    k.contains("|BLOCKED|")
                        && !k.starts_with("disp-assign|")
                        && !k.starts_with("disp-member-call|")
                })
                .take(24)
            {
                println!("    {count:>7}           {key}");
            }
            // The production axis. `prod-entered-untagged` is the tag-coverage
            // residue and is printed as a number on every scan: it is what the 37
            // tag sites in `body::shapes::mcall_{tail,chain,cmp}` have left to
            // explain, and inferring it from missing rows is exactly the mistake
            // this axis was built to stop.
            let residue = report.prod_untagged_residue();
            println!(
                "  member-call production first blocker (decode-only): {residue} bodies entered a \
                 production, declined, and reached NO tagged bail — the tag-coverage residue \
                 (target 0)"
            );
            for (key, count) in prodh.iter().filter(|(k, _)| !k.contains('|')) {
                let blocked = prodh
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<32}  blocked {blocked:>7}",
                    100.0 * *count as f64 / prod_seen.max(1) as f64
                );
            }
            for (key, count) in prodh
                .iter()
                .filter(|(k, _)| k.contains("|BLOCKED|"))
                .take(20)
            {
                println!("    {count:>7}           {key}");
            }
        }
    }

    for (class, title) in [
        (TuClass::CaptureFail, "top capture-fail reasons"),
        (TuClass::VocabGap, "top vocab gaps"),
        (TuClass::CodegenGap, "top codegen gaps"),
        (TuClass::PortError, "top port errors"),
        (TuClass::Mismatch, "mismatches"),
    ] {
        let reasons = report.top_reasons(class);
        if reasons.is_empty() {
            continue;
        }
        println!("\n  {title}:");
        for (reason, count) in reasons.iter().take(10) {
            println!("    {count:>5} x {reason}");
        }
        if reasons.len() > 10 {
            println!("    … and {} more distinct reasons", reasons.len() - 10);
        }
    }

    let mismatches = report.count(TuClass::Mismatch);
    if mismatches > 0 || diverged > 0 || report.cache.poisoned > 0 {
        eprintln!(
            "\nCORRECTNESS SIGNAL: {mismatches} mismatching TU(s), {diverged} replay \
             divergence(s), {} poisoned cache entr(ies)",
            report.cache.poisoned
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// `c2rs prefilter` — the reject-only pre-filter seam (see
/// [`c2_harness::prefilter`] for the contract that binds callers).
///
/// Prints exactly one line of JSON on stdout and exits 0 for every well-formed
/// verdict, including `not_implemented`. Exit 2 means "you called me wrong" —
/// a caller must treat that as a hard error, never as a verdict.
static PREFILTER_SPEC: Spec = Spec {
    cmd: "prefilter",
    opts: &[
        ("--source", Arity::Value),
        ("--flag", Arity::Repeated),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--emit-obj", Arity::Value),
        ("--compare-obj", Arity::Value),
        ("--obj-name", Arity::Value),
        ("--work", Arity::Value),
        ("--schema", Arity::Flag),
    ],
    requires: &[],
    max_positionals: 0,
};

fn cmd_prefilter(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&PREFILTER_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    // `--schema` used to SHORT-CIRCUIT mid-parse: it printed and returned from
    // inside the option loop, so every argument after it — including an unknown
    // one — was never examined. `prefilter --schema --typo` exited 0. The whole
    // command line is parsed first now, and only then does `--schema` win.
    if args.has("--schema") {
        println!("{}", prefilter::SCHEMA);
        return ExitCode::SUCCESS;
    }
    let source = args.get("--source").map(String::from);
    let mut flags: Vec<String> = args.all("--flag").into_iter().map(String::from).collect();
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    let emit_obj = args.path("--emit-obj");
    let compare_obj = args.path("--compare-obj");
    let obj_name = args.get("--obj-name").map(String::from);
    let work = args.path("--work");

    let Some(source) = source else {
        eprintln!(
            "usage: c2rs prefilter --source ARG (--flag F ... | --flags-file FILE) [--cwd DIR]\n\
             \x20                    [--emit-obj PATH] [--compare-obj PATH] [--obj-name Z:\\...]\n\
             \x20                    [--work DIR] | --schema\n\
             Prints one line of JSON; exit 0 = verdict, exit 2 = usage error.\n\
             Only verdict=\"reject\" licenses skipping a real compile."
        );
        return ExitCode::from(2);
    };

    if let Some(p) = &flags_file {
        match std::fs::read_to_string(p) {
            Ok(text) => {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    flags.extend(line.split_whitespace().map(String::from));
                }
            }
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", p.display());
                return ExitCode::from(2);
            }
        }
    }
    if flags.is_empty() {
        eprintln!("prefilter needs the TU's real compile flags (--flag / --flags-file)");
        return ExitCode::from(2);
    }

    let work = work.unwrap_or_else(|| scratch("prefilter"));
    let owned_work = work.clone();
    let req = prefilter::Request {
        source,
        flags,
        cwd,
        emit_obj,
        compare_obj,
        obj_name,
        work,
    };
    // `toolchain_quiet`, not `toolchain`: this command emits one line of JSON
    // and reports toolchain absence *inside* it, so a bare `SKIP:` line would
    // corrupt the output it is contracted to produce.
    let out = prefilter::run(args.toolchain_quiet().as_ref(), &req);
    println!("{}", out.to_json());
    // Captured IL bundles are large and this runs per candidate; the JSON (and
    // the emitted obj, which lives wherever the caller asked) is the record.
    let _ = std::fs::remove_dir_all(&owned_work);
    ExitCode::SUCCESS
}
