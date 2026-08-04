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
//! 3. **Board #195.** The fix for 2 was `cmd_capture`-only. `cmd_compile` kept
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
const C2RS: &str = env!("CARGO_BIN_EXE_c2rs");

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp/add3.cpp")
}

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "c2rs-cliflags-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
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
    // `census` is deliberately NOT in this list: it reads its `--flags-file`
    // AFTER `Toolchain::locate()` and does not refuse an empty one, so it still
    // carries this half of the class. Recorded rather than asserted, because
    // fixing it is a different function than board #195's.
    for sub in ["capture", "compile"] {
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
