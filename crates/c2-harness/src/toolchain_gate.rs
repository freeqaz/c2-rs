//! `c2-harness::toolchain_gate` — **the one place a command decides it cannot
//! grade**, and the one place that decision can be turned from a skip into a
//! failure.
//!
//! # What it replaces
//!
//! Fifteen hand-rolled `if !tc.has_strace() { println!("SKIP: …"); return
//! ExitCode::SUCCESS; }` blocks across six `cli/` modules, each with its own
//! copy of the string (`REFACTOR_REVIEW_2026-08-20.md` §2.2, lane `w-refrev`).
//! The duplication itself is minor. What is not minor is the second half of
//! §2.2's argument: **any demand mode has to be honoured at every one of those
//! sites independently**, so a demand added test-by-test is a demand with
//! fifteen holes in it. Funnelled, it is honoured everywhere by construction.
//!
//! # The two behaviours, and which one is the default
//!
//! **Default — unchanged, and it is a hard constraint (`CLAUDE.md`):** an absent
//! capability prints one `SKIP:` line and the command exits **0**. The lines are
//! byte-identical to the fifteen hand-written ones, because the form
//! `SKIP: <capabilities> absent (<why>)` is what all fifteen already used: they
//! name the capabilities the command *requires*, never the subset that happens
//! to be missing, so `perf` says `strace / i686-w64-mingw32-gcc absent` whenever
//! either is gone. That is preserved verbatim rather than "improved", because
//! five scripts scrape these lines.
//!
//! **On demand — `C2RS_REQUIRE_TOOLCHAIN` set to anything but `0`/empty:** the
//! same condition is a **failure**, on stderr, with a distinct word. The caller
//! has claimed this run grades; a green exit over a run that graded nothing is
//! the defect the variable exists to close (boards #3219 / #3226 / #3247, and
//! `crates/c2-harness/tests/require_toolchain.rs`, which is the same demand at
//! the *suite* level).
//!
//! The tension with `CLAUDE.md`'s *"the `c2rs` CLI must degrade cleanly when the
//! toolchain is absent — never panic/fail"* is stated rather than papered over,
//! and the boundary is exact: `Args::toolchain`'s `SKIP: toolchain absent` — the
//! line `CLAUDE.md` names — is **not** touched by this module and still exits 0
//! under every environment. What can fail here is a *partially* provisioned run
//! (a toolchain that located, minus `strace` / mingw / `c1xx.dll`), and only
//! when the caller has explicitly said the run is expected to grade. Board
//! #3338 named that gap in exactly those words when it closed the suite-level
//! half: *"a partially provisioned run … still skips a subset silently"*.
//!
//! # Why the demand rule lives here and not in the test that reads it
//!
//! `require_toolchain.rs` is an integration test of this crate, so it consumes
//! [`toolchain_demand`] rather than re-parsing the variable. One rule, one
//! implementation — which is the whole subject of §2.2 and would be a poor thing
//! to violate while fixing it.

use std::process::ExitCode;

use c2_reference::Toolchain;

/// The variable a caller sets to say *"this run is expected to grade against
/// real `c2.dll`"*. Any value but `0` and the empty string means yes.
/// PROV[N] not load-bearing — the environment variable name a caller sets to demand a real-toolchain run. A harness control, not a c2 fact.
pub const REQUIRE_TOOLCHAIN_VAR: &str = "C2RS_REQUIRE_TOOLCHAIN";

/// What the caller said about whether this run must grade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Demand {
    /// Variable unset — the default, and the portable lane's entitlement.
    Unset,
    /// Set to `0` or empty: the caller explicitly turned the demand off. Kept
    /// distinct from `Unset` so a `--portable` run says so in the log instead of
    /// being indistinguishable from a forgotten variable.
    Disabled(String),
    /// Set: this run claims to grade, and an absence is a failure.
    Demanded(String),
}

/// The rule itself, over a value rather than over the environment, so it is
/// testable without `set_var` (which is global to a test binary and races every
/// other test in it).
pub fn classify_demand(value: Option<&str>) -> Demand {
    match value {
        None => Demand::Unset,
        Some(v) if v.is_empty() || v == "0" => Demand::Disabled(v.to_string()),
        Some(v) => Demand::Demanded(v.to_string()),
    }
}

/// Read [`REQUIRE_TOOLCHAIN_VAR`]. The single implementation of the rule.
pub fn toolchain_demand() -> Demand {
    classify_demand(std::env::var(REQUIRE_TOOLCHAIN_VAR).ok().as_deref())
}

/// A toolchain capability a command needs beyond `cl.exe`/`c2.dll` themselves.
///
/// The label is the exact word the fifteen hand-written skip lines used; it is
/// part of a scraped stdout contract, not a description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    /// `strace` — the only way a capture keeps the `_CL_*` IL bundle.
    Strace,
    /// `i686-w64-mingw32-gcc` — builds the `c2host`/`c1host` stubs.
    Mingw,
    /// `c1xx.dll` — the C++ front end, for the standalone-c1 replay.
    C1xx,
}

impl Cap {
    /// The word this capability is called in a skip line.
    pub fn label(self) -> &'static str {
        match self {
            Cap::Strace => "strace",
            Cap::Mingw => "i686-w64-mingw32-gcc",
            Cap::C1xx => "c1xx.dll",
        }
    }

    /// Is it there?
    pub fn present(self, tc: &Toolchain) -> bool {
        match self {
            Cap::Strace => tc.has_strace(),
            Cap::Mingw => tc.has_mingw(),
            Cap::C1xx => tc.has_c1xx(),
        }
    }
}

/// The skip line for a set of required capabilities. Public so a test can
/// compare against the string a command will print without running it.
pub fn skip_line(need: &[Cap], why: &str) -> String {
    let labels: Vec<&str> = need.iter().map(|c| c.label()).collect();
    format!("SKIP: {} absent ({})", labels.join(" / "), why)
}

/// **The funnel.** `None` means every capability in `need` is present and the
/// caller should carry on. `Some(code)` means the caller must return `code`
/// immediately: the line has already been printed (or the refusal reported).
///
/// `why` is the parenthetical the call site owns — *"needed to keep the IL
/// bundle"*, *"needed for `--replay-every`"* — and it is the only per-site
/// string left.
pub fn toolchain_ready(tc: &Toolchain, need: &[Cap], why: &str) -> Option<ExitCode> {
    if need.iter().all(|c| c.present(tc)) {
        return None;
    }
    let what = skip_line(need, why);
    match toolchain_demand() {
        Demand::Demanded(v) => {
            // Deliberately does NOT contain the word `SKIP`: five scripts treat
            // a `SKIP` on this stream as "toolchain absent, the row is vacuous,
            // exit 0" (`scripts/mode_lane.sh:64`), which is the opposite of what
            // a refusal means.
            eprintln!(
                "c2rs: REFUSED — {REQUIRE_TOOLCHAIN_VAR}={v:?} is set, so this run CLAIMS \
                 to grade against real `c2.dll`, and it cannot: {what_body}. Skipping here \
                 would exit 0 over a run that graded nothing — the defect the variable \
                 exists to close (boards #3219 / #3226 / #3247). Unset the variable, or \
                 run `scripts/partest.sh --portable`, to allow the skip.",
                what_body = what.trim_start_matches("SKIP: "),
            );
            Some(ExitCode::FAILURE)
        }
        Demand::Unset | Demand::Disabled(_) => {
            println!("{what}");
            Some(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fifteen strings the `cli/` sites printed before the funnel, verbatim
    /// from the tree at `826ba1e413`. If the funnel's rendering ever stops
    /// reproducing one of them, this test says which — the scripts that scrape
    /// these lines have no other guard.
    #[test]
    fn every_converted_skip_line_is_byte_identical_to_the_string_it_replaced() {
        let cases: [(&[Cap], &str, &str); 8] = [
            (
                &[Cap::Strace],
                "needed to keep the IL bundle",
                "SKIP: strace absent (needed to keep the IL bundle)",
            ),
            (
                &[Cap::Strace],
                "needed to keep IL bundles during capture",
                "SKIP: strace absent (needed to keep IL bundles during capture)",
            ),
            (
                &[Cap::Mingw],
                "needed for --replay-every",
                "SKIP: i686-w64-mingw32-gcc absent (needed for --replay-every)",
            ),
            (
                &[Cap::Mingw],
                "needed to build c2host",
                "SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)",
            ),
            (
                &[Cap::Mingw],
                "needed to build the c1host stub",
                "SKIP: i686-w64-mingw32-gcc absent (needed to build the c1host stub)",
            ),
            (
                &[Cap::C1xx],
                "front end not located",
                "SKIP: c1xx.dll absent (front end not located)",
            ),
            (
                &[Cap::Strace, Cap::Mingw],
                "needed for standalone-c2 replay",
                "SKIP: strace / i686-w64-mingw32-gcc absent (needed for standalone-c2 replay)",
            ),
            (
                &[Cap::Strace, Cap::Mingw],
                "needed for replay",
                "SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)",
            ),
        ];
        for (need, why, expected) in cases {
            assert_eq!(
                skip_line(need, why),
                expected,
                "the funnel no longer reproduces a skip line that scripts scrape"
            );
        }
    }

    /// The demand rule, read at its values. Exercises [`classify_demand`] —
    /// the function `toolchain_demand` is a one-line wrapper around — rather
    /// than re-typing the match here: a test that re-types the rule its subject
    /// implements is a test of two spellings.
    #[test]
    fn the_demand_reads_unset_disabled_and_demanded_as_three_distinct_states() {
        let classify = classify_demand;
        assert_eq!(classify(None), Demand::Unset);
        assert_eq!(classify(Some("")), Demand::Disabled(String::new()));
        assert_eq!(classify(Some("0")), Demand::Disabled("0".to_string()));
        assert_eq!(classify(Some("1")), Demand::Demanded("1".to_string()));
        assert_eq!(classify(Some("yes")), Demand::Demanded("yes".to_string()));
        // `0` is the ONLY numeric off-switch: `00` is a value, not a zero, and
        // reading it as one would make `--portable`'s contract depend on a
        // parse. Pinned because the test suite's sentinel uses the same rule.
        assert_eq!(classify(Some("00")), Demand::Demanded("00".to_string()));
    }
}
