//! **`C2RS_REQUIRE_TOOLCHAIN` — the caller's way to demand that the workspace
//! suite actually graded something.** Lane `w-calleeguard`, taking
//! `w-mutcensus` **F1**.
//!
//! ## The defect this closes, stated as the numbers that carried it
//!
//! `cargo test --workspace --release` reads **1,660 / 0 / 43** in a provisioned
//! worktree and **1,660 / 0 / 43** in one with no `compilers/` — byte-identical,
//! because every toolchain-driven test prints `SKIP: toolchain absent` and
//! **passes** by design (`CLAUDE.md`: integration tests must degrade cleanly,
//! never panic). `w-mutcensus` §7 measured exactly that pair, with the
//! differential at **84.17 s** in one and **0.00 s** in the other, and its
//! prereg's own `targets != 42` invalidation rule was blind to it. Peer
//! `w-fence163` hit the same thing independently the same wave (board **#3219**,
//! **#3226**). **Two registered REDs read GREEN**, with a clean suite, the right
//! target count and the right exit code.
//!
//! `scripts/gate.sh` has `--require-graded` for precisely this failure, and
//! `grep -rn 'REQUIRE_TOOLCHAIN\|REQUIRE_GRADED' crates scripts` returned **8
//! hits, all in `scripts/gate.sh`, none under `crates/`** — while the workspace
//! suite row is quoted as evidence in essentially every rung doc in
//! `docs/rungs/`.
//!
//! ## Why it lands here and not in the lane that found it
//!
//! `w-mutcensus` F1 says it *"NOT TAKEN, and the reason is structural rather
//! than a shortage of time"*: it lands a test under `crates/`, and a
//! characterization lane's success criterion is a required-zero byte delta on
//! `crates fixtures scripts` — *"the same commit's two halves"*. **That is twice
//! in two waves that the instrument a lane discovered it needed could not be
//! landed by the lane that discovered it.**
//!
//! This lane's deliverable *is* `#[cfg(test)]` code under
//! `crates/c2-harness/`, so the conflict does not exist here. It is the same
//! observation `2026-08-16-guards.md` §8 item 5 makes from the other side: a
//! lane that lands tests cannot claim a graded-tree identity, and a lane that
//! claims one cannot land a test.
//!
//! ## The contract
//!
//! * **Default behaviour does not move.** With `C2RS_REQUIRE_TOOLCHAIN` unset —
//!   which is every ordinary run and the whole portable lane — this test passes
//!   and says so. The portable lane is entitled to be empty; the demand belongs
//!   to the **caller**, exactly as `gate.sh`'s `--require-graded` header argues.
//! * With it set to anything other than `0`, a run in which
//!   `Toolchain::locate()` returns `None` **fails**, and the failure message
//!   names the two numbers that would otherwise be identical.
//! * It is a **positive check on a name**, not an enumeration of the ways a run
//!   can be empty — `gate.sh`'s own design rule, quoted because the same design
//!   applies.

use c2_reference::Toolchain;

/// The variable a caller sets to say *"this run is expected to grade against
/// real `c2.dll`"*. Any value but `0` (and the empty string) means yes.
const VAR: &str = "C2RS_REQUIRE_TOOLCHAIN";

#[test]
fn a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with() {
    let demand = match std::env::var(VAR) {
        Err(_) => {
            println!(
                "{VAR} unset: this run makes no claim to have graded anything. \
                 That is the default and the portable lane's entitlement — set \
                 {VAR}=1 to make a toolchain-less run FAIL here instead of \
                 passing with a full green suite."
            );
            return;
        }
        Ok(v) => v,
    };
    if demand.is_empty() || demand == "0" {
        println!("{VAR}={demand:?}: demand explicitly disabled.");
        return;
    }

    let located = Toolchain::locate();
    assert!(
        located.is_some(),
        "{VAR}={demand:?} was set, so this run CLAIMS to have graded against \
         real `c2.dll` — and `Toolchain::locate()` returned None, so it graded \
         NOTHING. Every capture-driven test in this workspace printed `SKIP: \
         toolchain absent` and PASSED, and cargo swallows that line for a \
         passing test, so the totals are preserved: `w-mutcensus` §7 measured \
         1,648/0/42 with the differential at 84.17s and 1,648/0/42 with it at \
         0.00s. A probe defined by a count cannot detect a population that \
         silently left the count. Fix the environment (a fresh `git worktree \
         add` has no `compilers/` — it is gitignored and does not follow a new \
         worktree; use `scripts/setup_worktree.sh` or \
         `scripts/configure_existing_worktree.sh`), or unset {VAR} and stop \
         claiming this run graded."
    );
    // Positive on content, never on absence: say which toolchain answered.
    let tc = located.expect("just asserted");
    println!(
        "{VAR}={demand:?}: toolchain located — cl `{}`, c2 `{}`.",
        tc.cl_exe.display(),
        tc.c2_dll.display()
    );
}
