//! **The test that would have caught W-OXFIX.**
//!
//! On 2026-08-09 `c2rs selftest` exited non-zero on master with 319 PASS and 2
//! ERROR, and `scripts/status.sh` rendered a named gate row RED plus two
//! dependent rows `NO-RESULT`. The cause was one line of arithmetic nobody had
//! ever written down: `all_fixtures()` hands **every** `fixtures/cpp/*.cpp` to
//! `oracle_selftest`, which compiled all of them at `CAPTURE_IL_DEFAULT_FLAGS`
//! — and `wmain_no_return.cpp` / `wmain_no_return_neg.cpp`, whose whole point is
//! a non-`void` function with no `return` statement, are `error C4716` at every
//! flag word this toolchain has. Nothing asserted the corpus-wide invariant, so
//! it broke at the moment a fixture first violated it and was discovered by a
//! red gate row rather than by a test.
//!
//! The invariant, stated once and asserted here:
//!
//! > **Every fixture in `fixtures/cpp/` compiles at the profile `selftest` will
//! > use for it** — its declared `// c2rs-profile:` line, or
//! > `CAPTURE_IL_DEFAULT_FLAGS` when it declares none.
//!
//! Three lanes, and the split matters:
//!
//! * [`every_fixture_declares_a_wellformed_profile_or_none`] and
//!   [`the_default_is_unchanged_for_every_non_declaring_fixture`] are
//!   **portable** — no toolchain, so they run in `cargo test --workspace` on any
//!   machine and are what keeps a malformed marker from reaching a gate.
//! * [`every_fixture_compiles_at_the_profile_selftest_will_use`] needs the real
//!   toolchain and is the one that reproduces the defect. It **fails on master's
//!   tree** (both wmain fixtures, `error C4716`) and passes here.
//!
//! **There is no skip lane and there must not be one.** A fixture that cannot
//! compile is a named failure listing the fixture, its profile, and where the
//! profile came from. An opt-out that silently drops a fixture from the corpus is
//! the "absence read as success" shape (ROADMAP §9.18.8) — it would have made
//! this defect invisible instead of merely red, which is strictly worse.

use std::path::PathBuf;

use c2_harness::all_fixtures;
use c2_harness::fixture_profile::{resolve_profile, FixtureProfile, PROFILE_MARKER};
use c2_reference::{Toolchain, CAPTURE_IL_DEFAULT_FLAGS};

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::unique_scratch_dir("fxprof", tag)
}

fn name_of(p: &std::path::Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Portable. Every declaration in the corpus parses, or the corpus is broken.
///
/// This is the lane that catches a half-edited copy-paste (`PROFILE_MARKER`
/// twice), a marker with no reason, and a profile with no `/c` — before any of
/// them reaches a machine with a toolchain.
#[test]
fn every_fixture_declares_a_wellformed_profile_or_none() {
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found — this test would be vacuous");

    let mut declared: Vec<(String, String, String)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for cpp in &fixtures {
        match resolve_profile(cpp) {
            Ok(p) if p.declared => {
                declared.push((name_of(cpp), p.flags_str(), p.reason.clone()))
            }
            Ok(_) => {}
            Err(e) => errors.push(e.to_string()),
        }
    }
    assert!(errors.is_empty(), "malformed fixture profile(s):\n  {}", errors.join("\n  "));

    // Reported, not asserted: the declaring set is small and each entry carries
    // its own reason, so it reads as a list of justified deviations rather than
    // as a skip list. Asserting the *membership* would make every future lane
    // that needs a profile edit this test, which is how a rule stops meaning
    // anything; asserting well-formedness is what actually protects the corpus.
    println!(
        "fixture profiles: {} of {} fixtures declare one",
        declared.len(),
        fixtures.len()
    );
    for (n, f, why) in &declared {
        println!("  {n:<34} [{f}]\n      why: {why}");
    }
}

/// Portable, and this is the **"the default must not change" proof by count**.
///
/// Every fixture that declares nothing resolves to `CAPTURE_IL_DEFAULT_FLAGS`,
/// element for element — so the mechanism is a widening and not a change, and
/// the count of fixtures graded exactly as before is printed rather than
/// assumed.
#[test]
fn the_default_is_unchanged_for_every_non_declaring_fixture() {
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let want: Vec<String> = CAPTURE_IL_DEFAULT_FLAGS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        FixtureProfile::default_profile().flags,
        want,
        "the resolved default drifted from CAPTURE_IL_DEFAULT_FLAGS"
    );

    let (mut default_count, mut declared_count) = (0usize, 0usize);
    for cpp in &fixtures {
        let p = resolve_profile(cpp).unwrap_or_else(|e| panic!("{e}"));
        if p.declared {
            declared_count += 1;
            assert!(!p.reason.is_empty(), "{}: declared profile with no reason", name_of(cpp));
        } else {
            default_count += 1;
            assert_eq!(
                p.flags,
                want,
                "{} declares no `{PROFILE_MARKER}` yet did not resolve to the default — \
                 the default MUST be unchanged for every non-declaring fixture",
                name_of(cpp)
            );
        }
    }
    assert_eq!(
        default_count + declared_count,
        fixtures.len(),
        "every fixture resolves to exactly one profile"
    );
    println!(
        "profile resolution: {default_count} at the default [{}], {declared_count} declared, \
         {} total",
        want.join(" "),
        fixtures.len()
    );
}

/// **The regression test.** Needs the toolchain; skips cleanly without it
/// (CLAUDE.md's hard constraint), and a skip prints how many fixtures went
/// ungraded so an absent toolchain cannot read as a green corpus.
///
/// Fails on master's tree with both `wmain_no_return*.cpp` naming `error
/// C4716`; passes once they declare `/w14716`.
#[test]
fn every_fixture_compiles_at_the_profile_selftest_will_use() {
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");
    let Some(tc) = Toolchain::locate() else {
        eprintln!(
            "SKIP: toolchain absent — {} fixture(s) NOT compile-checked",
            fixtures.len()
        );
        return;
    };

    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for cpp in &fixtures {
        let profile = match resolve_profile(cpp) {
            Ok(p) => p,
            Err(e) => {
                failures.push(e.to_string());
                continue;
            }
        };
        let w = work("compile");
        let out = w.join("probe.obj");
        match tc.compile_obj_flags(cpp, &out, &profile.flags) {
            Ok(_) => checked += 1,
            Err(e) => failures.push(profile.compile_failure(cpp, "compile", &e)),
        }
        let _ = std::fs::remove_dir_all(&w);
    }

    assert!(
        failures.is_empty(),
        "{} of {} fixture(s) do not compile at the profile `selftest` will use for them. \
         This is the W-OXFIX defect: `c2rs selftest` will report ERROR on each of these and \
         exit non-zero, taking a named gate row RED. Fix the fixture, or declare a profile \
         it CAN compile at with `{PROFILE_MARKER} <flags…> # <why>` — never a skip list.\n\n{}",
        failures.len(),
        fixtures.len(),
        failures.join("\n\n")
    );
    println!("{checked} of {} fixture(s) compile at their resolved profile", fixtures.len());
}
