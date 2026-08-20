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
//!
//! The subcommand **handlers** live in [`cli`], one module per command group
//! (lane `w-mod`, size only). What stays here is the dispatch `match`, the usage
//! text, and — load-bearing — `mod argv`: the binary's one argument parser and,
//! through `Args::toolchain`, its one producer of a `Toolchain`.

use std::process::ExitCode;

use c2_reference::Toolchain;

mod cli;

use cli::census::cmd_census;
use cli::corpus::{cmd_corpus_gen, cmd_corpus_sample, cmd_corpus_stats};
use cli::factors::cmd_factors;
use cli::gap::cmd_gap;
use cli::listing::{cmd_listing, cmd_listing_scan};
use cli::perf::{cmd_perf, cmd_perf_scale};
use cli::prefilter::cmd_prefilter;
use cli::reference::{
    cmd_bench, cmd_capture, cmd_compile, cmd_diff, cmd_replay, cmd_replay_c1, cmd_selftest,
};
use cli::retrieve::{cmd_retrieve_eval, cmd_retrieve_index};
use cli::stage::cmd_stage;
use cli::search::{
    cmd_search_eval, cmd_search_from_lifter, cmd_search_from_retrieval, cmd_search_solve,
};

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
        "factors" => cmd_factors(rest),
        "listing" => cmd_listing(rest),
        "listing-scan" => cmd_listing_scan(rest),
        "prefilter" => cmd_prefilter(rest),
        "retrieve" => cmd_retrieve(rest),
        "search" => cmd_search(rest),
        "stage" => cmd_stage(rest),
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
         \x20 c2rs factors [opts]       the Phase-7 A/B/C/D/E sets as SETS: re-derive every published\n\
         \x20                           count from `gap --factors-tsv`, and intersect that listing\n\
         \x20                           with another lane's per-TU set (offline, no toolchain)\n\
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
         \x20            [--fnbyte-diff-jsonl PATH]  one JSON row per fnbyte-differs\n\
         \x20            FUNCTION: alignment, per-field diff class, relocation sites\n\
         \x20            (docs/DIFF_STRUCTURE.md; render with scripts/fndiff_report.py)\n\
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

pub(crate) use argv::{Args, Arity, Spec};

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
