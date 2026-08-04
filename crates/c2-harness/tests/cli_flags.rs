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
/// whole binary. Board #13 moved them into `src/cli/`, and a lint still pointed
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
/// **Widened for board #13.** The handlers moved from `main.rs` into
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
/// **Widened for board #13** for the same reason as the lint above: the nine
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

/// **The widening control.** Every invocation `scripts/` actually makes must
/// still be accepted — a parser that refuses everything would pass every test
/// above and break the gate.
///
/// These are toolchain-gated only in the sense that they may print
/// `SKIP: toolchain absent`; what is asserted is that they do **not** exit 2,
/// i.e. the parse accepted them. Needs no toolchain.
#[test]
fn every_invocation_the_scripts_make_is_still_accepted() {
    let cpp = s(&fixture());
    let w = work("accepted");
    let ff = write_flags(&w, "flags.txt", &["/Ox", "/GS-", "/c"]);
    let list = w.join("list.txt");
    std::fs::write(&list, "fixtures/cpp/add3.cpp\n").unwrap();
    let l = s(&list);
    let cases: Vec<Vec<&str>> = vec![
        vec!["selftest"],
        vec!["selftest", cpp.as_str()],
        vec!["bench"],
        vec!["perf"],
        vec!["diff", cpp.as_str()],
        vec!["census", cpp.as_str()],
        vec!["census", cpp.as_str(), "--flags-file", ff.as_str(), "--cwd", "."],
        vec!["capture", cpp.as_str()],
        vec!["compile", cpp.as_str()],
        vec!["gap", "--list", l.as_str(), "--flags-file", ff.as_str(), "--limit", "1", "--jobs", "1"],
        vec!["prefilter", "--schema"],
    ];
    for argv in &cases {
        let out = run(argv);
        assert_ne!(
            out.status.code(),
            Some(2),
            "`c2rs {}` is an invocation `scripts/` makes and the parser REFUSED it. \
             The seam is a widening plus a set of refusals for arguments that were being \
             dropped; it must not narrow a working command line.\nstderr:\n{}",
            argv.join(" "),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    let _ = std::fs::remove_dir_all(&w);
}
