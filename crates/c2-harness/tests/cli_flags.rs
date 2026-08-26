//! **A CLI flag path must not silently drop flags.**
//!
//! This is the **third** bug of one class, which is why it gets a test rather
//! than only a fix.
//!
//! 1. `c2rs compile` accepted `--flag` — which belongs to `c2rs listing` — and
//!    ignored it. A `/GR` vs `/GR-` probe therefore ran **two literally
//!    identical command lines**, produced identical objs, and the identity was
//!    read as a *finding about RTTI* rather than as a broken instrument.
//! 2. `c2rs capture` had no `--flags-file` at all and hard-coded `/Ox /GS- /c`
//!    ([`c2_reference::CAPTURE_IL_DEFAULT_FLAGS`]). Every `.gl` captured for
//!    analysis was taken at `/Ox` while the obj it was read against had been
//!    compiled at the workload's `/O1 /Oi /EHsc /GR …`. `/Ox` does **not** imply
//!    `/GF`, which is exactly the skew `gl_string_comdat_names` exists to catch.
//! 3. **Board #195, and then the whole class.** The fix for 2 was
//!    `cmd_capture`-only, and a sweep of all **26** dispatch arms afterwards
//!    found the same shape at **fourteen** more. `mod argv` in `main.rs` is the
//!    structural answer; the tests at the end of this file check the class
//!    rather than its instances, and one of them derives its table **from the
//!    dispatch `match` in the source** so a subcommand added later is covered
//!    the moment it is dispatched. `cmd_compile` kept
//!    three `iter().position(|a| a == "--x")` scans — the *original* site of bug
//!    1 — so `c2rs compile <cpp> --flag /GR-` still accepted and dropped
//!    `--flag`, an empty `--flags-file` still degraded to `cl.exe`'s own
//!    defaults, `--cwd` alone was parsed and then never consumed, and every
//!    argument check happened *after* `Toolchain::locate()` so a malformed
//!    invocation exited 0 on a machine with no compilers.
//!
//! All three have the same signature: **two different commands produce identical
//! output.** That is indistinguishable, at the terminal, from a real negative
//! result — so nothing about it looks like a bug. The halves below are the ways
//! to catch it:
//!
//! * [`an_unknown_option_is_refused_not_ignored`] — **needs no toolchain**. An
//!   option a command does not know must exit non-zero, never be scanned past.
//!   This is what would have caught bug 1 at the moment it was typed.
//! * [`an_empty_flags_file_is_refused`] / [`a_cwd_without_a_profile_is_refused`]
//!   — also toolchain-free. A profile that is *accepted and then not used* is
//!   the same failure one layer down.
//! * [`two_profiles_must_not_produce_one_bundle`] — toolchain-gated. Capture the
//!   *same* source at two profiles that provably differ and require the bytes to
//!   differ. This is what would have caught bug 2.
//! * [`two_profiles_must_not_produce_one_obj`] — the same shape on the **obj**
//!   side, for bug 3. Note it compares `.text` COMDAT bytes and not whole objs,
//!   for the reason given on the test: a whole-obj compare here is vacuous.
//! * [`the_default_profile_is_unchanged_by_the_widening`] — the control for the
//!   fix itself: `--flags-file <the default>` must be **byte-identical** to no
//!   `--flags-file`. Adding the option was a widening, not a change.
//!
//! **Must-fail discipline.** Two assertions in this file are *guarded* by
//! another: an exit-code check runs before the message check, and a decode check
//! runs before a byte comparison. Seeing the file go red is therefore not
//! evidence that the assertion you care about works — the mutation has to hold
//! the guard's quantity fixed (keep exiting 2, change only the message) so the
//! inner assertion is the one that fires, and each must name a *distinct*
//! failure. Every assertion message here is distinct on purpose.
//!
//! Toolchain-gated tests print `SKIP: toolchain absent` and pass, per the
//! CLAUDE.md hard constraint.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use c2_obj::ObjImage;
use c2_reference::{Toolchain, CAPTURE_IL_DEFAULT_FLAGS};

/// The `c2rs` binary cargo just built for this test — never a stale
/// `target/release/c2rs` off disk (`scripts/harness_bin.sh` documents what a
/// stale harness binary costs).
/// PROV[N] not load-bearing — `env!("CARGO_BIN_EXE_c2rs")`, the path cargo builds this crate's own binary at. Supplied by the build system.
const C2RS: &str = env!("CARGO_BIN_EXE_c2rs");

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp/add3.cpp")
}

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::clean_scratch_dir("cliflags", tag)
}

fn run(args: &[&str]) -> Output {
    Command::new(C2RS)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("cannot run {C2RS}: {e}"))
}

/// The captured `.ex` in a `--keep-il` directory. The bundle base is a
/// content hash that changes with the profile, so the file is found by
/// **suffix**, not by name — and there must be exactly one.
fn kept_ex(dir: &Path) -> Vec<u8> {
    let mut hits: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("no --keep-il dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "ex").unwrap_or(false))
        .collect();
    assert_eq!(hits.len(), 1, "expected exactly one .ex in {}", dir.display());
    std::fs::read(hits.pop().unwrap()).unwrap()
}

fn write_flags(dir: &Path, name: &str, flags: &[&str]) -> String {
    let p = dir.join(name);
    std::fs::write(&p, format!("{}\n", flags.join(" "))).unwrap();
    p.to_string_lossy().into_owned()
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// A short content tag for failure messages. `assert_eq!` on two 3 KB
/// `Vec<u8>`s prints both in full and buries the sentence that says what broke —
/// this is an FNV-1a over the bytes, which is enough to say "same" or "different"
/// at a glance. std only, no dependency.
fn digest(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("fnv1a={h:016x}")
}

/// The first offset at which two bundles disagree, rendered with a small window.
fn first_diff(a: &[u8], b: &[u8]) -> String {
    match a.iter().zip(b).position(|(x, y)| x != y) {
        None => format!(", identical prefix; lengths {} vs {}", a.len(), b.len()),
        Some(i) => format!(", first differing byte at {i}: {:02x} vs {:02x}", a[i], b[i]),
    }
}

/// **No toolchain needed.** An option the command does not understand must be
/// *refused*, not scanned past.
///
/// `c2rs capture` used to find its one option with an
/// `iter().position(|a| a == "--keep-il")` scan, which ignores every other
/// argument by construction — pass `--flags-file work/dc3-workload/flags.txt`
/// and it compiled at `/Ox` anyway and said nothing. Exit code **2** is the
/// contract (`ExitCode::from(2)` = usage error), and it is checked *before*
/// `Toolchain::locate()` so this runs on a machine with no compilers at all.
///
/// The list is a table on purpose: a subcommand added later that scans instead
/// of parsing is one line away from being covered here.
#[test]
fn an_unknown_option_is_refused_not_ignored() {
    let cpp = s(&fixture());
    // (subcommand, an option that subcommand does NOT define)
    let cases: &[(&str, &str)] = &[
        // `--flag` belongs to `listing`. This is bug 1's exact argument.
        ("capture", "--flag"),
        ("capture", "--keep-obj"),
        ("census", "--flag"),
        ("census", "--keep-obj"),
        // Board #195. `compile` is where bug 1 actually happened, and it kept
        // its three `position()` scans through `6a33b4d` — the fix there was
        // `capture`-only. `--flag` is the literal argument that made the `/GR`
        // vs `/GR-` probe run two identical command lines; `--keep-il` belongs
        // to `capture`/`census` and is the cross-command direction.
        ("compile", "--flag"),
        ("compile", "--keep-il"),
    ];
    for &(sub, opt) in cases {
        let out = run(&[sub, cpp.as_str(), opt, "/GR-"]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {sub} <cpp> {opt} /GR-` must exit 2, got {:?}. An accepted-and-dropped \
             flag makes two different commands produce identical output.\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        // A DISTINCT assertion with a DISTINCT message, because exiting 2 and
        // saying which argument caused it are two different properties and a
        // reader debugging a red run needs to know which one broke. Note this
        // one is only reachable when the line above passes — see the must-fail
        // note on this module.
        assert!(
            err.contains(opt),
            "`c2rs {sub}` refused correctly but ANONYMOUSLY: the message must NAME the \
             option it refused ({opt}), or the user cannot tell which of several \
             arguments was rejected; got: {err}"
        );
    }
}

/// **No toolchain needed.** An option that is *parsed* but only *consumed* on
/// another code path is dropped just as silently as one that is scanned past.
///
/// `cmd_compile` reads `--cwd` into a variable that is used only inside the
/// `--flags-file` branch: `compile_obj` makes the source absolute and
/// translates it to `Z:\…` itself, so `c2rs compile <cpp> --cwd <dir>` compiled
/// something other than what was asked for and said nothing. Refusing the
/// combination is the only outcome distinguishable from honouring it.
///
/// Every `--cwd` in `scripts/` and `docs/` pairs it with `--flags-file`, so
/// this refusal costs no existing invocation.
#[test]
fn a_cwd_without_a_profile_is_refused() {
    let cpp = s(&fixture());
    let out = run(&["compile", cpp.as_str(), "--cwd", "/nonexistent-dir"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "`c2rs compile <cpp> --cwd DIR` with no --flags-file must exit 2, got {:?}. \
         --cwd is consumed only on the --flags-file path, so accepting it alone \
         compiles at a different cwd than the one named.\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--cwd"),
        "the refusal must NAME --cwd as the argument that has no effect; got: {err}"
    );
}

/// **No toolchain needed.** `--flags-file` with nothing usable in it must be
/// refused rather than degrading to `cl.exe`'s own defaults — the same
/// dropped-profile failure one layer down.
#[test]
fn an_empty_flags_file_is_refused() {
    let w = work("emptyflags");
    let ff = write_flags(&w, "empty.txt", &["# only a comment"]);
    let cpp = s(&fixture());
    // `census` IS in this list now. It used to read its `--flags-file` *after*
    // `Toolchain::locate()` and refuse nothing, so an all-comment profile fell
    // back to `cl.exe`'s own defaults and the `/Gy`-dependent census/gate
    // cross-check was reported against a profile nobody named — and none of it
    // was reachable without a toolchain, so no test here could have seen it.
    for sub in ["capture", "compile", "census"] {
        let out = run(&[sub, cpp.as_str(), "--flags-file", ff.as_str()]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {sub} <cpp> --flags-file <empty>` must be refused, not silently \
             replaced by cl.exe's defaults.\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let _ = std::fs::remove_dir_all(&w);
}

/// Two profiles that differ must produce two different `.ex` bundles.
///
/// `/Ox` vs `/Od` is chosen because the per-function optimization word is
/// recorded *in the IL* — the port reads it (`opt_mode_of_word`) and refuses
/// anything unmodeled — so the two captures cannot legitimately coincide. If
/// they do, the CLI dropped the profile.
#[test]
fn two_profiles_must_not_produce_one_bundle() {
    if Toolchain::locate().is_none() {
        eprintln!("SKIP: toolchain absent");
        return;
    }
    let w = work("twoprofiles");
    let cpp = s(&fixture());

    let ox = write_flags(&w, "ox.txt", &["/Ox", "/GS-", "/c"]);
    let od = write_flags(&w, "od.txt", &["/Od", "/GS-", "/c"]);
    let dir_ox = w.join("keep-ox");
    let dir_od = w.join("keep-od");

    for (ff, dir) in [(&ox, &dir_ox), (&od, &dir_od)] {
        let keep = s(dir);
        let out = run(&[
            "capture",
            cpp.as_str(),
            "--flags-file",
            ff.as_str(),
            "--keep-il",
            keep.as_str(),
        ]);
        assert!(
            out.status.success(),
            "capture at {ff} failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        // The command must also SAY which profile it used — a silent flag is
        // half of what made this class of bug invisible.
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(
            said.contains("profile:"),
            "capture must print the profile it used; got:\n{said}"
        );
    }

    let a = kept_ex(&dir_ox);
    let b = kept_ex(&dir_od);
    assert!(
        a != b,
        "`--flags-file /Ox …` and `--flags-file /Od …` produced BYTE-IDENTICAL IL \
         ({} B, {}). Two different compile profiles cannot yield one bundle: the CLI is \
         accepting --flags-file and dropping it, which is the bug this test exists for.",
        a.len(),
        digest(&a),
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// The control for the fix: naming the default explicitly must be
/// **byte-identical** to not naming it. Adding `--flags-file` to `capture` was a
/// widening, and this is what says so.
///
/// It also pins the *path* handling, not just the flag list: with no `--cwd` the
/// source argument is still made absolute and translated to `Z:\…` exactly as
/// `Toolchain::capture_il` has always done, so the `.gl` and `.debug$S` record
/// the same name. A regression there changes these bytes even though the flags
/// match.
#[test]
fn the_default_profile_is_unchanged_by_the_widening() {
    if Toolchain::locate().is_none() {
        eprintln!("SKIP: toolchain absent");
        return;
    }
    let w = work("default");
    let cpp = s(&fixture());
    let ff = write_flags(&w, "default.txt", &CAPTURE_IL_DEFAULT_FLAGS);
    let implicit = w.join("keep-implicit");
    let explicit = w.join("keep-explicit");
    let (imp, exp) = (s(&implicit), s(&explicit));

    let out = run(&["capture", cpp.as_str(), "--keep-il", imp.as_str()]);
    assert!(
        out.status.success(),
        "default capture failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run(&[
        "capture",
        cpp.as_str(),
        "--flags-file",
        ff.as_str(),
        "--keep-il",
        exp.as_str(),
    ]);
    assert!(
        out.status.success(),
        "explicit-default capture failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let (a, b) = (kept_ex(&implicit), kept_ex(&explicit));
    assert!(
        a == b,
        "`capture <cpp>` and `capture <cpp> --flags-file <{}>` must be byte-identical; \
         the option is a widening, not a change. implicit {} B {}, explicit {} B {}{}",
        CAPTURE_IL_DEFAULT_FLAGS.join(" "),
        a.len(),
        digest(&a),
        b.len(),
        digest(&b),
        first_diff(&a, &b),
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// Board #195, the obj side of [`two_profiles_must_not_produce_one_bundle`]:
/// two `c2rs compile` profiles that differ must produce two different objs.
///
/// **A whole-obj compare would be VACUOUS here and is not what this asserts.**
/// `cmd_compile` writes through a per-invocation scratch directory
/// (`c2rs-cli-compile-<pid>-<nanos>-<n>`) and `cl.exe` bakes that `/Fo` path
/// into `.debug$S`, so *any* two runs differ by ~14 bytes and by the
/// `TimeDateStamp` regardless of the flags — `assert!(a != b)` on the raw bytes
/// would pass just as happily with the profile dropped. (Measured: the default
/// profile and an explicit `/Ox /GS- /c` `--flags-file` differ in exactly those
/// 14 path bytes plus the timestamp, and in nothing else. That is also why
/// `compile` gets no byte-identity control of the kind
/// [`the_default_profile_is_unchanged_by_the_widening`] gives `capture`.)
///
/// So the comparison is on the **`.text` COMDAT bytes**, which contain no path
/// and no timestamp and which `/Ox` vs `/Od` cannot legitimately leave equal.
#[test]
fn two_profiles_must_not_produce_one_obj() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        // The `--flags-file` path goes through `capture_reference_with`, which
        // needs strace to keep the `_CL_*` bundle alive.
        eprintln!("SKIP: strace absent");
        return;
    }
    let w = work("twoobjs");
    // The `--flags-file` path passes the source argument to `cl.exe`
    // **verbatim** (build-faithful — a project TU's relative path is what gets
    // baked into the obj), so this is spelled the way `c2rs gap` spells it: a
    // repo-relative source under an explicit `--cwd`. An absolute host path
    // here is rejected by `cl.exe` itself, loudly.
    let root = s(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let cpp = "fixtures/cpp/add3.cpp";
    // `/Gy` is in both profiles and is not the variable under test: without
    // function-level linking `cl.exe` emits one non-COMDAT `.text`, which
    // `text_comdat_functions_with_bytes` fail-closes on — and a comparison over
    // an empty list is exactly the vacuous pass this file exists to prevent.
    // The only difference between the two profiles is `/Ox` vs `/Od`.
    let ox = write_flags(&w, "ox.txt", &["/Ox", "/GS-", "/Gy", "/c"]);
    let od = write_flags(&w, "od.txt", &["/Od", "/GS-", "/Gy", "/c"]);
    let ox_obj = w.join("ox.obj");
    let od_obj = w.join("od.obj");

    let mut code: Vec<Vec<(String, Vec<u8>)>> = Vec::new();
    for (ff, dest) in [(&ox, &ox_obj), (&od, &od_obj)] {
        let keep = s(dest);
        let out = run(&[
            "compile",
            cpp,
            "--flags-file",
            ff.as_str(),
            "--cwd",
            root.as_str(),
            "--keep-obj",
            keep.as_str(),
        ]);
        assert!(
            out.status.success(),
            "compile at {ff} failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        // The command must also SAY which profile it used — a silent flag is
        // half of what made this class of bug invisible.
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(
            said.contains("profile:"),
            "compile must print the profile it used; got:\n{said}"
        );
        let bytes = std::fs::read(dest)
            .unwrap_or_else(|e| panic!("no kept obj at {}: {e}", dest.display()));
        let img = ObjImage::new(bytes);
        let fns = img.text_comdat_functions_with_bytes().unwrap_or_default();
        assert!(
            !fns.is_empty(),
            "obj from {ff} decoded to zero .text COMDATs ({} B) — the comparison below \
             would be vacuous, which is the failure mode this whole file exists for",
            img.len(),
        );
        code.push(fns);
    }

    let (a, b) = (&code[0], &code[1]);
    let same = a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|((na, ba), (nb, bb))| na == nb && ba == bb);
    assert!(
        !same,
        "`--flags-file /Ox …` and `--flags-file /Od …` produced IDENTICAL .text COMDAT \
         bytes ({} functions, {}). Two different compile profiles cannot yield one \
         codegen: the CLI is accepting --flags-file and dropping it, which is the bug \
         this test exists for.",
        a.len(),
        digest(&a.iter().flat_map(|(_, v)| v.iter().copied()).collect::<Vec<u8>>()),
    );
    let _ = std::fs::remove_dir_all(&w);
}

// ===========================================================================
// The CLASS, not the instances — `mod argv` and the sweep behind it
// ===========================================================================

/// `crates/c2-harness/src/main.rs`, for the tests that read the source itself.
fn main_rs() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// **Every source file of the `c2rs` binary**, as `(display-name, contents)`:
/// `src/main.rs` plus every `.rs` under `src/cli/`.
///
/// The handlers used to live in `main.rs`, so a lint that read one file read the
/// whole binary. Lane `w-mod` moved them into `src/cli/`, and a lint still pointed
/// at `main.rs` alone would have kept passing while covering almost none of the
/// code it exists to constrain — absence reading as success. Every source lint
/// over the binary takes its file set from here instead.
fn bin_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = vec![("src/main.rs".to_string(), main_rs())];
    let cli = root.join("cli");
    let entries = std::fs::read_dir(&cli)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", cli.display()));
    let mut files: Vec<std::path::PathBuf> = entries
        .map(|e| e.expect("cannot read a src/cli entry").path())
        .filter(|p| p.extension().map(|x| x == "rs").unwrap_or(false))
        .collect();
    files.sort();
    for p in files {
        let text = std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
        out.push((format!("src/cli/{}", p.file_name().unwrap().to_string_lossy()), text));
    }
    // A directory listing that comes back empty would make every lint below
    // vacuously green while measuring one file — the exact failure mode the
    // widening exists to prevent. `mod.rs` plus the ten handler modules is the
    // floor; the count only ever grows.
    assert!(
        out.len() >= 11,
        "bin_sources() found only {} file(s) ({:?}). The `c2rs` binary is main.rs plus \
         the src/cli/ modules; a short listing makes every source lint in this file \
         measure a fraction of the binary and still pass.",
        out.len(),
        out.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    out
}

/// Strip `//` line comments so a source lint reads code, not prose about code.
/// (`://` is left alone so a URL survives.)
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) if i > 0 && l.as_bytes()[i - 1] == b':' => l,
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Every subcommand the dispatcher knows must refuse an unknown option.**
///
/// The table is **derived from `main.rs`'s dispatch `match`**, not written out
/// here. That is the difference between "I fixed the ones I found" and "these
/// are the last ones": a hand-written list proves completeness only over itself,
/// and a subcommand added next week would not be in it. This one covers a new
/// arm the moment it is dispatched.
///
/// Needs no toolchain — the refusal is required to happen *before*
/// `Args::toolchain`, which is the whole point of the seam.
#[test]
fn every_subcommand_refuses_an_unknown_option() {
    let src = code_only(&main_rs());
    // `"name" => cmd_handler(` — the dispatch arms, top level and sub-dispatch.
    let mut cmds: Vec<(String, String)> = Vec::new();
    for line in src.lines() {
        let l = line.trim();
        let Some(q0) = l.strip_prefix('"') else { continue };
        let Some(qe) = q0.find('"') else { continue };
        let name = &q0[..qe];
        let after = &q0[qe + 1..];
        let Some(arrow) = after.find("=> cmd_") else { continue };
        let handler: String = after[arrow + 3..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() || name.starts_with('-') {
            continue;
        }
        cmds.push((name.to_string(), handler));
    }
    // A parser that matched nothing and a CLI with no subcommands look identical
    // from the outside — ROADMAP §9.18.8's shape. Pin the count's floor.
    assert!(
        cmds.len() >= 26,
        "the dispatch extractor found only {} arms; it is broken, and a broken \
         extractor makes this whole test vacuously green. Found: {cmds:?}",
        cmds.len()
    );

    // Sub-dispatched arms need their group prefix (`corpus` + `gen`).
    let group_of = |handler: &str| -> Option<&'static str> {
        for g in ["corpus", "retrieve", "search"] {
            if handler.starts_with(&format!("cmd_{g}_")) {
                return Some(match g {
                    "corpus" => "corpus",
                    "retrieve" => "retrieve",
                    _ => "search",
                });
            }
        }
        None
    };

    let mut checked = 0usize;
    for (name, handler) in &cmds {
        // The three group dispatchers reject an unknown SUBCOMMAND, which is a
        // different message; their leaves are covered individually below.
        if matches!(handler.as_str(), "cmd_corpus" | "cmd_retrieve" | "cmd_search") {
            continue;
        }
        let mut argv: Vec<&str> = Vec::new();
        if let Some(g) = group_of(handler) {
            argv.push(g);
        }
        argv.push(name);
        // The unknown option goes FIRST, with no positional at all. Two reasons,
        // both learned from this test failing on its own first run: `Args::parse`
        // runs to completion before any handler checks its positionals, so a
        // missing `<cpp>` cannot mask the refusal; and a trailing positional is
        // itself refused by the commands that take none (`bench`, `perf`, `gap`),
        // which made the *surplus-positional* guard fire first and left the
        // assertion this test exists for unreached. A probe that trips an earlier
        // guard measures the earlier guard.
        argv.push("--definitely-not-an-option");
        let out = run(&argv);
        checked += 1;
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {}` must exit 2, got {:?}. An option a subcommand does not define \
             must be REFUSED, never scanned past: an accepted-and-dropped flag makes two \
             different commands produce one output, which is indistinguishable at the \
             terminal from a real negative result.\nstdout:\n{}\nstderr:\n{}",
            argv.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        // A DISTINCT assertion: exiting 2 and saying which argument caused it are
        // two different properties. Guarded by the one above — see the must-fail
        // note in this module's header.
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--definitely-not-an-option"),
            "`c2rs {}` refused correctly but ANONYMOUSLY: the message must NAME the \
             option it refused, or a user with several arguments cannot tell which one \
             was rejected; got: {err}",
            argv.join(" "),
        );
    }
    assert!(
        checked >= 23,
        "only {checked} subcommands were actually exercised; the skip list is eating the test"
    );
}

/// **`Toolchain::locate` may appear in the whole `c2rs` binary only inside
/// `mod argv`, which lives in `main.rs`.**
///
/// This is the structural half of the fix, checked rather than trusted. Eight
/// handlers used to call the free `located()` as their *first* statement, so a
/// malformed command line exited **0** on a machine with no compilers — a usage
/// error the binary never reported, and therefore one no test could pin. With
/// `Args::toolchain(&self)` the only producer, a handler cannot reach a
/// toolchain until it holds a parsed argument set.
///
/// **Widened by lane `w-mod`.** The handlers moved from `main.rs` into
/// `src/cli/*.rs`. Scanning `main.rs` alone would then have covered the parser
/// and almost nothing else — every handler, i.e. every site the eight defects
/// were actually found at, would have become free to call `Toolchain::locate`
/// directly with this lint still green. That is precisely the "a second producer
/// becomes expressible" failure the seam forbids, so the file set is now the
/// whole binary ([`bin_sources`]) and only the byte range of `mod argv` inside
/// `main.rs` is exempt. Strictly stronger: same rule, more files.
///
/// A convention nobody checks is how this class reached fourteen sites.
#[test]
fn locate_is_reachable_only_through_the_arg_seam() {
    let mut total_hits = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    let mut seam_hits = 0usize;

    for (name, text) in bin_sources() {
        let src = code_only(&text);
        // `mod argv` is in main.rs and nowhere else; every other file of the
        // binary is exempt from nothing.
        let seam = if name == "src/main.rs" {
            let start = src
                .find("mod argv {")
                .expect("mod argv is gone from main.rs — the seam has been removed");
            let end = src
                .find("use argv::{")
                .expect("the `use argv::{...}` that closes the module is gone");
            assert!(
                start < end,
                "module bounds inverted; this lint is not measuring what it thinks"
            );
            Some(start..end)
        } else {
            assert!(
                !src.contains("mod argv {"),
                "{name} declares a second `mod argv`. There is exactly one argument parser \
                 in this binary and it lives in main.rs; a second one re-opens the class."
            );
            None
        };

        for (i, _) in src.match_indices("Toolchain::locate") {
            total_hits += 1;
            match &seam {
                Some(r) if r.contains(&i) => seam_hits += 1,
                _ => offenders.push(format!("{name}:{}", src[..i].matches('\n').count() + 1)),
            }
        }
    }

    // If the extractor finds nothing, the lint is vacuous — pin the floor.
    assert!(
        total_hits > 0,
        "no `Toolchain::locate` anywhere in the c2rs binary: this lint is measuring nothing"
    );
    assert!(
        seam_hits > 0,
        "`Toolchain::locate` exists in the binary but not once inside `mod argv`: the \
         producer has moved out of the seam and this lint's exemption range is measuring \
         nothing"
    );
    assert!(
        offenders.is_empty(),
        "`Toolchain::locate` is called outside `mod argv` at {offenders:?}. \
         The seam exists so that \"parse and validate, THEN locate\" is the only \
         expressible order; a direct call re-opens the ordering defect, where a bogus \
         command line exits 0 with `SKIP: toolchain absent` exactly where the portable \
         test lane runs. Use `Args::toolchain()` / `Args::toolchain_quiet()`."
    );
}

/// **`opt()` — the scan helper — must stay dead.**
///
/// It was `iter().position(|a| a == key)`, i.e. boards #194/#195's bug wearing a
/// helper's name, and nine handlers used it. Deleting it is what made the class
/// unreachable; re-adding one is how it would come back.
///
/// **Widened by lane `w-mod`** for the same reason as the lint above: the nine
/// callers now live in `src/cli/*.rs`, so a `main.rs`-only scan would have left
/// every one of their sites unwatched. The file set is the whole binary
/// ([`bin_sources`]) — same rule, more files.
#[test]
fn the_position_scan_helper_is_not_reintroduced() {
    for (name, text) in bin_sources() {
        let src = code_only(&text);
        assert!(
            !src.contains("position(|a| a =="),
            "{name} contains a `position(|a| a == ...)` argument scan again. A scan cannot \
             refuse what it does not look for, so every other argument is invisible by \
             construction. Add the option to the subcommand's `Spec` instead."
        );
    }
}

/// **An argument that is accepted and then never consumed must be refused.**
///
/// Every row is a real dangling option found by the sweep: parsed into a
/// variable that some code path simply ignores. They are one class with the
/// `--cwd` case boards #194/#195 named, and they are checked together because
/// fixing them one at a time is what produced three commands with the identical
/// dangling `--cwd` and only one refusal.
///
/// Needs no toolchain.
#[test]
fn an_accepted_but_unconsumed_argument_is_refused() {
    // (argv, the substring the message must contain)
    let cases: &[(&[&str], &str)] = &[
        // `--cwd` is consumed only on the `--flags-file` path. THREE commands had
        // it and only `compile` refused it.
        (&["capture", "x.cpp", "--cwd", "/tmp"], "--cwd"),
        (&["census", "x.cpp", "--cwd", "/tmp"], "--cwd"),
        (&["compile", "x.cpp", "--cwd", "/tmp"], "--cwd"),
        // `--query-div` is read only on the held-out path.
        (&["retrieve", "eval", "d", "--split", "loo", "--query-div", "3"], "--query-div"),
        // Order-dependent contradiction: `--cache X --no-cache` dropped X, while
        // `--no-cache --cache X` used X.
        (&["gap", "--list", "a", "--flags-file", "b", "--cache", "X", "--no-cache"], "--no-cache"),
        (&["gap", "--list", "a", "--flags-file", "b", "--no-cache", "--cache", "X"], "--no-cache"),
        // Nothing to validate with no cache — it printed the validation line anyway.
        (
            &["gap", "--list", "a", "--flags-file", "b", "--no-cache", "--validate-cache", "5"],
            "--validate-cache",
        ),
        // An empty profile falls back to cl.exe's own defaults.
        // (covered for capture/compile above; census had no such refusal at all)
    ];
    for (argv, needle) in cases {
        let out = run(argv);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {}` must exit 2, got {:?}. This argument is parsed and then never \
             consumed on the path it selects, so accepting it runs something other than \
             what was asked for — in silence.\nstdout:\n{}\nstderr:\n{}",
            argv.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(needle),
            "`c2rs {}` refused correctly but did not NAME {needle}; a refusal that does \
             not identify the argument leaves the user guessing. got: {err}",
            argv.join(" "),
        );
    }
}

/// **A numeric option must refuse a value that is not a number.**
///
/// `opt(...).and_then(|s| s.parse().ok())` turned a typo into the default in
/// silence. The sharpest instance: `search from-lifter --compiles abc` left
/// `Budget::default()`'s 400 instead of the bounded 200 the handler intends —
/// a typo doubled the compile budget and nothing said so.
///
/// Needs no toolchain.
#[test]
fn a_numeric_option_refuses_a_non_number() {
    let cases: &[(&[&str], &str)] = &[
        (&["perf", "--port-iters", "abc"], "--port-iters"),
        (&["perf", "--ref-iters", "abc"], "--ref-iters"),
        (&["perf-scale", "--port-secs", "abc"], "--port-secs"),
        // A partly-bad list silently dropped the bad element and ran the rest.
        (&["perf-scale", "--conc", "1,x,4"], "--conc"),
        (&["gap", "--list", "a", "--flags-file", "b", "--jobs", "eight"], "--jobs"),
        (&["gap", "--list", "a", "--flags-file", "b", "--limit", "lots"], "--limit"),
        (&["listing-scan", "--list", "a", "--flags-file", "b", "--jobs", "eight"], "--jobs"),
        (&["corpus", "gen", "--seed", "abc"], "--seed"),
        (&["search", "eval", "--compiles", "abc"], "--compiles"),
        (&["retrieve", "eval", "d", "--k", "1,x,10"], "--k"),
    ];
    for (argv, needle) in cases {
        let out = run(argv);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {}` must exit 2, got {:?}. A numeric option whose value does not parse \
             used to become the DEFAULT in silence, so the run used a number nobody \
             chose.\nstdout:\n{}\nstderr:\n{}",
            argv.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(needle),
            "`c2rs {}` refused a bad number but did not NAME {needle}; got: {err}",
            argv.join(" "),
        );
    }
}

/// **An option with a fixed value set must refuse anything outside it.**
///
/// A `_ =>` arm that swallows every unrecognised spelling is the dropped-flag
/// failure in another costume. `retrieve eval --split heldout` ran leave-one-out's
/// *opposite* and reported `held-out`; `search eval --moves lenght` ran the full
/// moveset while **echoing `moves=lenght`** in its own header — a report naming a
/// configuration it did not run.
///
/// Needs no toolchain.
#[test]
fn an_enumerated_option_refuses_a_value_outside_its_set() {
    let cases: &[(&[&str], &str)] = &[
        (&["retrieve", "eval", "d", "--split", "heldout"], "--split"),
        (&["search", "eval", "--moves", "lenght"], "--moves"),
        (&["search", "solve", "x.cpp", "--moves", "short"], "--moves"),
    ];
    for (argv, needle) in cases {
        let out = run(argv);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {}` must exit 2, got {:?}. An unrecognised value fell through a `_ =>` \
             arm to the default, and the header then reported the string the user \
             typed.\nstdout:\n{}\nstderr:\n{}",
            argv.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(needle),
            "`c2rs {}` refused but did not NAME {needle}; got: {err}",
            argv.join(" "),
        );
    }
}

/// **Structural refusals the scans could not express**, and one the dispatcher
/// never even saw. Needs no toolchain.
#[test]
fn the_parser_refuses_what_a_scan_could_not_see() {
    let cases: &[(&[&str], &str)] = &[
        // `bench` was dispatched as `cmd_bench()`, so its arguments were dropped
        // by the DISPATCHER — one level above any handler that could refuse them.
        (&["bench", "--jobs", "4"], "--jobs"),
        // A repeated single-valued option: first-wins meant `--list a --list c`
        // ran against `a` while the terminal showed `c`.
        (&["gap", "--list", "a", "--flags-file", "b", "--list", "c"], "--list"),
        // A missing value at end of line silently became "option absent".
        (&["capture", "x.cpp", "--keep-il"], "--keep-il"),
        // A value that is itself an option: `--seed --count 5` made the seed the
        // literal string "--count".
        (&["corpus", "gen", "--seed", "--count"], "--seed"),
        // A surplus positional: `corpus sample --out /tmp/x` wrote the sample
        // into a directory literally named `--out`.
        (&["corpus", "sample", "--out", "/tmp/x"], "--out"),
        // `prefilter --schema` returned from INSIDE the option loop, so anything
        // after it was never examined.
        (&["prefilter", "--schema", "--typo"], "--typo"),
        // `search solve --d` was advertised by the top-level usage and never read.
        (&["search", "solve", "x.cpp", "--d", "3"], "--d"),
        // `census` had no empty-profile refusal at all.
        (&["diff", "x.cpp", "--flags-file", "f.txt"], "--flags-file"),
    ];
    for (argv, needle) in cases {
        let out = run(argv);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs {}` must exit 2, got {:?}.\nstdout:\n{}\nstderr:\n{}",
            argv.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(needle),
            "`c2rs {}` refused but did not NAME {needle}; got: {err}",
            argv.join(" "),
        );
    }
}

// ---- THE WIDENING CONTROL, AND WHY IT IS FOUR TESTS AND NOT ONE ---------------
//
// **Every invocation `scripts/` actually makes must still be accepted** — a
// parser that refuses everything would pass every test above and break the gate.
// These are toolchain-gated only in the sense that they may print
// `SKIP: toolchain absent`; what is asserted is that they do **not** exit 2,
// i.e. the parse accepted them. Needs no toolchain.
//
// This was ONE `#[test]` until 2026-08-08 (`every_invocation_the_scripts_make_
// is_still_accepted`) and it cost **116 s of the target's 119 s**, because three
// of its eleven invocations are whole-corpus commands run to completion —
// `selftest` 44 s, `bench` 43 s, `perf` 29 s — executed **serially inside one
// test** in order to assert that their argv parses. `cargo test` runs the 36
// test binaries serially, so that one test was ~1/3 of the whole `cargo test
// --workspace --release` leg of every merge.
//
// **Nothing about what is checked has changed.** The command lines are the same
// eleven, the assertion is the same `assert_ne!(code, Some(2))` with the same
// message, and the three expensive commands still run to completion. The only
// change is that they are in four `#[test]`s instead of one, so the default
// parallel test harness overlaps them: the target's wall goes to ~max(44 s)
// instead of sum(116 s).
//
// **The split is a PARTITION OF ONE ROSTER, not four hand-copied lists.** Four
// literal lists would be four places for an invocation to be dropped in, and a
// dropped invocation is a check that silently stops running — the failure mode
// `docs/GAPS.md` §7 records for lanes and STATUS.md's trap 5 records twelve
// times over. So there is one `scripts_invocation_roster()`, each row tagged
// with the group that runs it; `the_split_is_a_partition_of_the_roster` pins the
// roster's size and its group counts, and every group asserts it ran a non-zero
// number of invocations rather than passing vacuously on an empty filter.

// ---- AND WHAT THE THREE EXPENSIVE ONES WERE NOT ASSERTING (board #3337) -------
//
// Everything above is true and remained true, and it left the file paying ~155 s
// per suite run for three whole-corpus differential runs whose **verdicts reached
// no assertion at all**. `assert_ne!(code, Some(2))` is "the parser did not refuse
// this argv"; `cmd_bench` signals a corpus failure with `ExitCode::FAILURE` (= 1),
// and **1 != 2**, so the assertion passes over it. Named by
// `docs/REFACTOR_REVIEW_2026-08-20.md` §0.1 as one of three live instances of
// this repo's defining defect family — absence read as success — sitting inside
// the warranty layer itself.
//
// [`assert_corpus_verdict`] is the second assertion, on the **same** executions:
// zero added wall time, because the work is already being paid for.
//
// **THE REVIEW'S PROPOSED FIX WOULD NOT HAVE CAUGHT A WRONG EMIT, AND THIS IS THE
// PART WORTH READING.** §0.1 proposes `assert_eq!(code, Some(0))` on all three.
// Two facts, both read out of the source and then confirmed by planting a wrong
// emit and watching what happened (lane `w-warranty`, mutation M1):
//
//   1. **`c2rs perf` exits 0 on a port `Mismatch` DELIBERATELY.**
//      `cli/perf.rs`: *"the reference is the sole judge, so a port
//      Match/Mismatch/NotImplemented is per-TU reporting, not a harness failure.
//      Only a capture/replay error or a broken P0.1 replay is a hard failure of
//      the benchmark itself."* The `Port=Mismatch -> FAILURE` the review cites
//      lives in **`cmd_diff`**, the per-fixture command — not in `cmd_bench`.
//   2. **`bench` and `selftest` never invoke the port.** Both loop
//      `oracle_selftest`, which is *reference* determinism plus *reference*
//      capture stability. `cmd_bench`'s `fail == 0 && err == 0` is a statement
//      about `c2.dll`'s own reproducibility.
//
// So an exit-code-only guard would have caught reference instability and **read
// as though it caught wrong emits** — the same defect family one level up, and
// the reason this lane's rule was "every guard is PROVEN to fire, by mutation,
// before it is claimed". The corpus-wide port compare *is* computed, by `perf`,
// and printed on its `summary:` line, and then dropped on the floor. So the
// wrong-emit guard reads **stdout**, and `perf`'s exit-code contract is left
// exactly as it was.
//
// Three properties each guard has, because each is a recorded lesson:
//
//   * **Absence is keyed to a positive fact, never sniffed out of a string.**
//     The guard asks `Toolchain::locate()`, `has_strace()` and `has_mingw()`
//     directly and computes what the command was *obliged* to do, rather than
//     grepping stdout for the word `SKIP` — which would let any silent early
//     return read as a clean skip.
//   * **A short run is not a pass.** Every row count is compared against
//     `c2_harness::all_fixtures().len()`, read from the same source the command
//     reads, so a corpus that half-ran fails instead of reporting zero failures.
//   * **Two derivations, diffed** (board #3288). The mismatch count is taken
//     from the summary line *and* by counting per-fixture rows, and the two must
//     agree; ditto `bench`'s failure count.

/// Which of the four `#[test]`s below runs a given invocation. The three
/// whole-corpus commands get one each because each is ~30-45 s; everything else
/// is together because the eight of them together are seconds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Accepted {
    Selftest,
    Bench,
    Perf,
    Rest,
}

/// **The roster** — every invocation `scripts/` makes, in the order it stood in
/// when this was one test, each tagged with the group that runs it.
fn scripts_invocation_roster<'a>(
    cpp: &'a str,
    ff: &'a str,
    l: &'a str,
) -> Vec<(Accepted, Vec<&'a str>)> {
    use Accepted::*;
    vec![
        (Selftest, vec!["selftest"]),
        (Rest, vec!["selftest", cpp]),
        (Bench, vec!["bench"]),
        (Perf, vec!["perf"]),
        (Rest, vec!["diff", cpp]),
        (Rest, vec!["census", cpp]),
        (Rest, vec!["census", cpp, "--flags-file", ff, "--cwd", "."]),
        (Rest, vec!["capture", cpp]),
        (Rest, vec!["compile", cpp]),
        (
            Rest,
            vec!["gap", "--list", l, "--flags-file", ff, "--limit", "1", "--jobs", "1"],
        ),
        (Rest, vec!["prefilter", "--schema"]),
    ]
}

// ---------------------------------------------------------------------------
// The corpus-verdict guard (board #3337)
// ---------------------------------------------------------------------------

/// The three roster rows that run a whole-corpus differential and therefore
/// carry a verdict this file is entitled to assert on.
///
/// **Deliberately small, and deliberately a `match` on the WHOLE argv.** The
/// brief's caution is the right one: the roster is mixed, and a blanket
/// `exit == 0` over eleven rows would have to be softened until it fitted the
/// weakest of them. `c2rs diff` exits 1 on `Port=Mismatch` *on purpose*;
/// `census`, `capture`, `compile`, `gap` and `prefilter --schema` each have
/// their own contract, and none of them was read line-by-line by this lane. A
/// narrow assertion that fires beats a broad one that had to be weakened, so
/// the other eight rows keep the roster assertion and nothing more — stated
/// here rather than left as an accident of the `match` arms.
///
/// [`the_corpus_verdict_guard_covers_exactly_the_three_whole_corpus_rows`] pins
/// this set BY NAME, so a row that is renamed or given an argument silently
/// leaves coverage with something going red.
fn corpus_verdict_kind(argv: &[&str]) -> Option<CorpusVerdict> {
    match argv {
        ["selftest"] => Some(CorpusVerdict::Selftest),
        ["bench"] => Some(CorpusVerdict::Bench),
        ["perf"] => Some(CorpusVerdict::Perf),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CorpusVerdict {
    Selftest,
    Bench,
    Perf,
}

/// The whitespace-separated token immediately BEFORE `word`, parsed as a
/// number. Returns `None` when `word` does not appear or what precedes it is
/// not a number — and every caller treats that `None` as a FAILURE, never as a
/// pass, so a summary-line format change breaks loudly here instead of quietly
/// disarming the guard.
fn num_before(line: &str, word: &str) -> Option<u64> {
    let toks: Vec<&str> = line.split_whitespace().collect();
    let i = toks.iter().position(|t| *t == word)?;
    if i == 0 {
        return None;
    }
    toks[i - 1].parse().ok()
}

/// The `N` of a trailing `(of N)`.
fn num_after_of(line: &str) -> Option<usize> {
    let toks: Vec<&str> = line.split_whitespace().collect();
    let i = toks.iter().position(|t| *t == "(of")?;
    toks.get(i + 1)?.trim_end_matches(')').parse().ok()
}

/// The one `summary:` line, or `None`. More than one is itself a failure —
/// two summaries mean the caller is looking at output it does not understand.
fn the_summary_line(stdout: &str) -> Option<&str> {
    let mut hits = stdout.lines().filter(|l| l.trim_start().starts_with("summary:"));
    let first = hits.next()?;
    if hits.next().is_some() {
        return None;
    }
    Some(first)
}

/// Per-fixture report rows: a line whose first token is a `.cpp` file name.
/// `selftest`/`bench` (`selftest_row`) and `perf` all lead their rows with the
/// fixture's file name, and nothing else either command prints does — the
/// provenance header's first tokens are `c2-rs`, `binary`, `workload`, `wibo`,
/// `cl.exe`, `c2.dll`, `c1xx.dll`.
fn fixture_rows(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|l| {
            l.split_whitespace()
                .next()
                .map(|t| t.ends_with(".cpp"))
                .unwrap_or(false)
        })
        .collect()
}

/// The last whitespace-separated token of a row — `perf`'s port status column.
fn last_tok(line: &str) -> &str {
    line.split_whitespace().next_back().unwrap_or("")
}

/// **The second assertion.** Called on the same `Output` the roster assertion
/// just looked at, for the three rows [`corpus_verdict_kind`] admits. Returns
/// `true` if it asserted a graded verdict, `false` if it recorded a legitimate
/// absence — the caller counts both, so "the guard ran" and "the guard graded"
/// are separate facts and neither can stand in for the other.
fn assert_corpus_verdict(kind: CorpusVerdict, argv: &[&str], out: &Output) -> bool {
    let cmd = argv.join(" ");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let code = out.status.code();
    let ctx = || {
        format!(
            "\ncommand: `c2rs {cmd}`\nexit: {code:?}\n\
             --- last 40 lines of stdout ---\n{}\n--- stderr ---\n{}",
            stdout
                .lines()
                .rev()
                .take(40)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
            stderr,
        )
    };

    // ---- ABSENCE, DECIDED POSITIVELY -------------------------------------
    // Not by looking for the word SKIP in stdout. `Toolchain::locate()` is the
    // same call the CLI makes, so this branch is taken exactly when the CLI's
    // own `args.toolchain()` returned None. A run that reaches here graded
    // nothing and says so; `require_toolchain::a_run_that_claims_to_grade_must_
    // have_a_toolchain_to_grade_with` is what turns that into a failure when
    // the caller set `C2RS_REQUIRE_TOOLCHAIN`.
    let Some(tc) = Toolchain::locate() else {
        println!(
            "SKIP: toolchain absent — the corpus-verdict guard for `c2rs {cmd}` graded \
             NOTHING. Set C2RS_REQUIRE_TOOLCHAIN=1 to make this run FAIL instead."
        );
        return false;
    };
    // `perf` additionally needs strace + mingw for the standalone-c2 replay and
    // prints its own SKIP when they are missing. Same rule: computed from the
    // toolchain, not read off stdout.
    if kind == CorpusVerdict::Perf && (!tc.has_strace() || !tc.has_mingw()) {
        assert_eq!(
            code,
            Some(0),
            "CORPUS VERDICT GUARD: `c2rs perf` had no strace/mingw and so must have \
             skipped cleanly, and it exited {code:?} instead.{}",
            ctx()
        );
        println!(
            "SKIP: strace/mingw absent — the corpus-verdict guard for `c2rs {cmd}` graded NOTHING."
        );
        return false;
    }

    let want_rows = c2_harness::all_fixtures().len();
    assert!(
        want_rows > 0,
        "CORPUS VERDICT GUARD: `all_fixtures()` is EMPTY, so every row count below \
         would have a floor of zero and this guard would assert nothing. That is the \
         vacuous-pass shape, and it is a failure here.{}",
        ctx()
    );
    let rows = fixture_rows(&stdout);
    assert_eq!(
        rows.len(),
        want_rows,
        "CORPUS VERDICT GUARD: `c2rs {cmd}` reported {} per-fixture rows and the corpus \
         holds {want_rows}. A SHORT RUN IS NOT A PASS — this is the truncation floor, and \
         it is derived from `c2_harness::all_fixtures()`, the same source the command \
         itself reads.{}",
        rows.len(),
        ctx()
    );

    match kind {
        // ---- selftest: reference determinism + capture stability ----------
        // NOTE what this does NOT cover: `selftest` never invokes the port, so
        // no assertion here can see a wrong emit. That is `perf`'s row below.
        CorpusVerdict::Selftest => {
            let bad: Vec<&&str> = rows
                .iter()
                .filter(|l| !l.split_whitespace().any(|t| t == "PASS"))
                .collect();
            assert!(
                bad.is_empty(),
                "CORPUS VERDICT GUARD: `c2rs selftest` reported {} non-PASS fixture(s) — the \
                 ORACLE's own determinism or capture stability is broken, which invalidates \
                 every differential verdict taken against it. Rows:\n{}{}",
                bad.len(),
                bad.iter().take(20).map(|l| l.to_string()).collect::<Vec<_>>().join("\n"),
                ctx()
            );
            // Second derivation of the same fact (#3288): the exit code, which
            // `cmd_selftest` sets from `all_pass`. Disagreement is itself a bug.
            assert_eq!(
                code,
                Some(0),
                "CORPUS VERDICT GUARD: `c2rs selftest` printed {} PASS rows and then exited \
                 {code:?}. The row scan and the exit code are two derivations of one fact and \
                 they DISAGREE, so one of them is lying.{}",
                rows.len(),
                ctx()
            );
        }
        // ---- bench: same engine, summary-line renderer --------------------
        CorpusVerdict::Bench => {
            let sum = the_summary_line(&stdout).unwrap_or_else(|| {
                panic!(
                    "CORPUS VERDICT GUARD: `c2rs bench` printed no single `summary:` line. The \
                     line is this guard's whole subject; its absence is a FAILURE and never a \
                     pass.{}",
                    ctx()
                )
            });
            let fail = num_before(sum, "fail,").unwrap_or_else(|| {
                panic!(
                    "CORPUS VERDICT GUARD: could not read the fail count out of `{sum}`. The \
                     summary-line format moved and this guard can no longer grade it — which \
                     is a FAILURE, not a pass.{}",
                    ctx()
                )
            });
            let err = num_before(sum, "error").unwrap_or_else(|| {
                panic!(
                    "CORPUS VERDICT GUARD: could not read the error count out of `{sum}`.{}",
                    ctx()
                )
            });
            assert_eq!(
                (fail, err),
                (0, 0),
                "CORPUS VERDICT GUARD: `c2rs bench` ran the whole-corpus oracle self-test and \
                 reported {fail} fail, {err} error. The oracle is the sole judge of this port; \
                 if IT is not reproducible, nothing graded against it means anything. Summary \
                 line: `{sum}`{}",
                ctx()
            );
            assert_eq!(
                num_after_of(sum),
                Some(want_rows),
                "CORPUS VERDICT GUARD: `c2rs bench`'s summary says `{sum}` but the corpus holds \
                 {want_rows} fixtures.{}",
                ctx()
            );
            assert_eq!(
                code,
                Some(0),
                "CORPUS VERDICT GUARD: `c2rs bench` summarised {fail} fail / {err} error and \
                 then exited {code:?}. Summary line and exit code are two derivations of one \
                 fact and they DISAGREE.{}",
                ctx()
            );
        }
        // ---- perf: THE ONE THAT SEES A WRONG EMIT -------------------------
        //
        // `perf::bench_fixture` runs the port over every fixture and compares
        // its obj against the reference byte for byte. Until board #3337 the
        // resulting per-fixture `Mismatch` was printed and dropped: `cmd_perf`
        // returns SUCCESS on a port mismatch by design, and nothing read the
        // line. This is the corpus-wide byte gate finally reaching an assertion
        // inside `cargo test`.
        CorpusVerdict::Perf => {
            let sum = the_summary_line(&stdout).unwrap_or_else(|| {
                panic!(
                    "CORPUS VERDICT GUARD: `c2rs perf` printed no single `summary:` line. That \
                     line carries the corpus-wide port mismatch count and its absence is a \
                     FAILURE, not a pass.{}",
                    ctx()
                )
            });
            let summary_mismatch = num_before(sum, "mismatch,").unwrap_or_else(|| {
                panic!(
                    "CORPUS VERDICT GUARD: could not read the port mismatch count out of \
                     `{sum}`. The summary-line format moved and this guard can no longer see a \
                     wrong emit — which is a FAILURE, not a pass.{}",
                    ctx()
                )
            });
            // Second, differently-built derivation (#3288): count the rows
            // whose status column is a Mismatch. The summary and the rows are
            // produced by different code paths (`PerfReport::tally` vs the
            // per-row printer) and must agree.
            let row_mismatch: Vec<&&str> = rows
                .iter()
                .filter(|l| last_tok(l).starts_with("Mismatch"))
                .collect();
            assert_eq!(
                summary_mismatch as usize,
                row_mismatch.len(),
                "CORPUS VERDICT GUARD: `c2rs perf`'s summary says {summary_mismatch} mismatch \
                 and its own per-fixture rows show {}. Two derivations of one number, and they \
                 DISAGREE — the instrument is broken before the port is even judged.{}",
                row_mismatch.len(),
                ctx()
            );
            assert_eq!(
                summary_mismatch,
                0,
                "CORPUS VERDICT GUARD: **A WRONG EMIT.** `c2rs perf` compared the port's obj \
                 against real c2's, byte for byte, over all {want_rows} fixtures, and \
                 {summary_mismatch} of them DIFFER. A wrong emit scores strictly below the \
                 refusal it replaced (docs/PROGRESS_METRIC.md) — this is an alarm, not a gap. \
                 `NotImplemented` is not counted here and never should be. Mismatching \
                 fixtures:\n{}{}",
                row_mismatch
                    .iter()
                    .take(20)
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                ctx()
            );
            // `perf` returns FAILURE only for a capture/replay error or a broken
            // P0.1 replay (`ref-replay-inexact`) — NOT for a port mismatch,
            // which is why the assertion above reads stdout. Both of those are
            // real failures of the benchmark, so the exit code is worth having
            // too; it is simply not the wrong-emit catch.
            assert_eq!(
                code,
                Some(0),
                "CORPUS VERDICT GUARD: `c2rs perf` exited {code:?}. It returns FAILURE for a \
                 capture/replay ERROR row or a broken P0.1 reference replay \
                 (`[!ref-replay-inexact]`) — never for a port mismatch — so this is the oracle \
                 supply line, not the port.{}",
                ctx()
            );
        }
    }
    true
}

/// Run one group of the roster. The assertion is transcribed verbatim from the
/// single test this replaces, and it names its own command line, so a failure in
/// any group says which invocation the parser refused.
fn accepted_group(group: Accepted, tag: &str) {
    let cpp = s(&fixture());
    let w = work(tag);
    let ff = write_flags(&w, "flags.txt", &["/Ox", "/GS-", "/c"]);
    let list = w.join("list.txt");
    std::fs::write(&list, "fixtures/cpp/add3.cpp\n").unwrap();
    let l = s(&list);
    let mut ran = 0usize;
    let mut guarded = 0usize;
    let mut graded = 0usize;
    for (g, argv) in scripts_invocation_roster(&cpp, &ff, &l) {
        if g != group {
            continue;
        }
        let out = run(&argv);
        assert_ne!(
            out.status.code(),
            Some(2),
            "`c2rs {}` is an invocation `scripts/` makes and the parser REFUSED it. \
             The seam is a widening plus a set of refusals for arguments that were being \
             dropped; it must not narrow a working command line.\nstderr:\n{}",
            argv.join(" "),
            String::from_utf8_lossy(&out.stderr),
        );
        // THE SECOND ASSERTION, on the same execution (board #3337). Separately
        // named from the roster assertion above so a failure says which of the
        // two fired: the roster assertion is about argv, this one is about the
        // corpus verdict the run just produced and used to discard.
        if let Some(kind) = corpus_verdict_kind(&argv) {
            guarded += 1;
            if assert_corpus_verdict(kind, &argv, &out) {
                graded += 1;
            }
        }
        ran += 1;
    }
    // A group whose filter matches nothing would pass having checked nothing —
    // the split's own version of the absence-reads-as-success defect, and the
    // one thing splitting a test can newly break.
    assert!(
        ran > 0,
        "group {group:?} ran ZERO invocations. The roster no longer tags any row \
         with it, so this test passed without executing a single command line."
    );
    // The same shape one level down, for the guard rather than for the roster: a
    // heavy group whose expensive invocation stopped being *guarded* would still
    // run the corpus for 40 s and still assert only its argv, which is exactly
    // the state #3337 exists to end. Pinned per group, so a re-tagged row cannot
    // move the coverage without a named failure.
    let want_guarded = match group {
        Accepted::Selftest | Accepted::Bench | Accepted::Perf => 1,
        Accepted::Rest => 0,
    };
    assert_eq!(
        guarded, want_guarded,
        "group {group:?} applied the corpus-verdict guard to {guarded} invocation(s), \
         expected {want_guarded}. The three whole-corpus rows carry a verdict and the \
         other eight deliberately do not (see `corpus_verdict_kind`)."
    );
    println!(
        "group {group:?}: {ran} invocation(s) run, {guarded} carrying a corpus verdict, \
         {graded} of those actually GRADED (the rest recorded a toolchain absence)."
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// `c2rs selftest` — the oracle self-test over every fixture, ~44 s.
#[test]
fn every_invocation_the_scripts_make_is_still_accepted_selftest() {
    accepted_group(Accepted::Selftest, "accepted-selftest");
}

/// `c2rs bench` — the fixture gate, ~43 s.
#[test]
fn every_invocation_the_scripts_make_is_still_accepted_bench() {
    accepted_group(Accepted::Bench, "accepted-bench");
}

/// `c2rs perf` — the fixture gate plus the per-obj timing, ~29 s.
#[test]
fn every_invocation_the_scripts_make_is_still_accepted_perf() {
    accepted_group(Accepted::Perf, "accepted-perf");
}

/// The other eight invocations, which are seconds between them.
#[test]
fn every_invocation_the_scripts_make_is_still_accepted_rest() {
    accepted_group(Accepted::Rest, "accepted-rest");
}

/// **The split's own control.** The four tests above partition
/// [`scripts_invocation_roster`]; this pins the roster's size and each group's
/// share, so an invocation cannot be deleted, and a row cannot be re-tagged into
/// a group, without something going red. Needs no toolchain and spawns nothing.
#[test]
fn the_split_is_a_partition_of_the_roster() {
    let roster = scripts_invocation_roster("CPP", "FF", "L");
    assert_eq!(
        roster.len(),
        11,
        "the roster held 11 invocations when it was one test; it holds {}. \
         An invocation removed from here stops being checked at all, and nothing \
         else in this file would notice.",
        roster.len()
    );
    for (group, want) in [
        (Accepted::Selftest, 1),
        (Accepted::Bench, 1),
        (Accepted::Perf, 1),
        (Accepted::Rest, 8),
    ] {
        let got = roster.iter().filter(|(g, _)| *g == group).count();
        assert_eq!(
            got, want,
            "group {group:?} covers {got} invocations, expected {want}. \
             The four #[test]s are a partition of this roster; a re-tagged or \
             dropped row changes what runs without changing any test's name."
        );
    }
}

/// **The corpus-verdict guard's own control** (board #3337). Pins WHICH roster
/// rows are guarded, **by name and not by count**, for the reason
/// `docs/rungs/README.md` gives: a control pinned by count passes the moment the
/// count matches, whoever is in it. Needs no toolchain and spawns nothing, so it
/// is also the "did this binary build" control for the mutation campaign.
///
/// If a row is renamed, given an argument, or dropped, its guard silently stops
/// applying and the three expensive tests go back to asserting only their argv —
/// which is the exact state #3337 closes. That regression fails HERE, in a test
/// that takes microseconds, rather than nowhere.
#[test]
fn the_corpus_verdict_guard_covers_exactly_the_three_whole_corpus_rows() {
    let roster = scripts_invocation_roster("CPP", "FF", "L");
    let covered: Vec<(String, CorpusVerdict)> = roster
        .iter()
        .filter_map(|(_, argv)| corpus_verdict_kind(argv).map(|k| (argv.join(" "), k)))
        .collect();
    assert_eq!(
        covered,
        vec![
            ("selftest".to_string(), CorpusVerdict::Selftest),
            ("bench".to_string(), CorpusVerdict::Bench),
            ("perf".to_string(), CorpusVerdict::Perf),
        ],
        "the corpus-verdict guard covers {covered:?}. It must cover exactly the three \
         bare whole-corpus invocations — those are the ones that run the differential \
         over every fixture and used to discard the result."
    );
    // And the negative half, also by name: the eight rows that deliberately are
    // NOT guarded. Written out so that adding a row without deciding about it is
    // a failure rather than a default.
    let unguarded: Vec<String> = roster
        .iter()
        .filter(|(_, argv)| corpus_verdict_kind(argv).is_none())
        .map(|(_, argv)| argv[0].to_string())
        .collect();
    assert_eq!(
        unguarded,
        vec!["selftest", "diff", "census", "census", "capture", "compile", "gap", "prefilter"],
        "the UNGUARDED rows moved. Each of these was left with the roster assertion \
         alone on purpose (`c2rs diff` exits 1 on Port=Mismatch by design; the others' \
         exit contracts were not read line-by-line by the lane that added this guard). \
         Adding a row here is a decision, not a default: either extend \
         `corpus_verdict_kind` or extend this list and say why."
    );
}

/// **The census LADDER seam refuses a depth it cannot honour** (lane `w-joint3`,
/// board **#3506**+). No toolchain needed.
///
/// `c2rs census --relax N` drives the shipped [`c2_il::Relax`] ladder, and
/// `--tsv PATH` writes the per-**slot** dump a blocker ladder reads. Both are
/// instrument-only seams, and both are on the path where this repo's most
/// expensive failure family lives: **a run that quietly graded at a depth other
/// than the one asked for publishes a number against the wrong denominator.**
///
/// For a scan that is an obvious error. For a **ladder** it is not: every rung's
/// output is a judgement of the form *"did lifting this clause move anything?"*,
/// so a rung that silently ran at `STRICT` reads as **"lifting this clause
/// changed nothing"** — a substantive and completely wrong conclusion that looks
/// exactly like a real negative result. That is board **#3470**'s shape
/// (`SKIP: toolchain absent` exits 0 and grades nothing) one level up, and it is
/// why the parse is a hard refusal rather than a fallback.
///
/// Checked **without** a toolchain deliberately: `args.toolchain()` returns
/// `None` and exits 0 on a machine with no compilers, so a usage check placed
/// after it is one no portable lane can pin — the exact ordering bug
/// [`an_empty_flags_file_is_refused`] records for `--flags-file`.
#[test]
fn a_relax_depth_that_does_not_parse_is_refused_not_defaulted() {
    let cpp = s(&fixture());
    for bad in ["bogus", "", "-1", "1.5"] {
        let out = run(&["census", cpp.as_str(), "--relax", bad]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`c2rs census --relax {bad:?}` must exit 2. Falling back to STRICT would make a \
             ladder rung that graded at depth 0 indistinguishable from a clause that moved \
             nothing.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--relax"),
            "the refusal must NAME --relax as the argument it could not honour; got: {err}"
        );
    }
}

/// **The relaxation ladder has a stated depth, and a caller cannot silently
/// select one that does not exist.**
///
/// `Relax::level` saturates at the last defined level *by design* — so a
/// `--relax 99` is not an error. What must hold is that the ladder's advertised
/// arity and the enum's own `LEVELS` are the same number, because the refusal
/// message above quotes `LEVELS` as the legal range. A `LEVELS` that grew
/// without the message following it would tell an operator that a depth is
/// illegal when it is not — and an operator who believes that never runs the
/// rung.
#[test]
fn the_relax_ladder_names_every_level_it_has() {
    let names: Vec<&str> = (0..c2_il::Relax::LEVELS)
        .map(|n| c2_il::Relax::level(n).name())
        .collect();
    assert_eq!(
        names,
        vec!["strict", "name-from-gl"],
        "the shipped relaxation ladder changed. `c2rs census --relax N`'s refusal message \
         quotes `Relax::LEVELS` as the legal range and `scripts/joint_ladder.py` treats \
         level 1 as the whole post-parse NAME family; both need re-reading if this moves."
    );
    assert_eq!(
        c2_il::Relax::level(0),
        c2_il::Relax::STRICT,
        "level 0 must BE the incumbent census, or the ladder's rung 0 is not a control"
    );
    assert_ne!(
        c2_il::Relax::level(1),
        c2_il::Relax::STRICT,
        "level 1 must DIFFER from level 0, or the ladder's first rung is green by \
         construction — trap 0's stronger form (#3454): ask what the control would do if \
         the effect were total"
    );
}
