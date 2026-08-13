// w-fencecount — THE POSITIVE CONTROL FOR THE `fence-blocks-exact` COUNTER.
//
// The counter this file grades (`GapReport::fence_blocks`, printed by every
// `c2rs gap` scan as the FENCE-BLOCKS-EXACT block and as `gap-metric fence-*`)
// answers CLAUDE.md's two-sided fence-pricing question one fence at a time:
// **how many TUs does this fence alone hold out of `match`, and how many of
// those are already byte-exact in every emitted body?** The second number is
// the expensive one, and `src/xdk/LIBCMT/vsnprnc.cpp` is why it exists:
// `docs/rungs/2026-08-09-w-vsnprnc.md` §1 found both its functions byte-exact
// against real c2 with the TU still `vocab-gap`, held out by the inline fence
// and nothing else — a whole TU of refusal no per-function instrument could
// see and nothing standing counted.
//
// **ON THE 878-TU WORKLOAD THAT COUNTER READS ZERO, WHICH IS WHY THIS FILE
// EXISTS.** `w-fence2` (board #2470) paid vsnprnc's fence and converted the TU,
// so the workload's one instance of the shape is gone; all 845 held TUs today
// carry at least two decode causes (1,716 firings over 845 TUs), so no cause is
// any TU's SOLE blocker and every `sole`/`exact` cell is 0. A counter whose only
// positive reading has already been paid off is indistinguishable from one that
// is not wired up — this file is the cell that keeps it honest, built out of
// nothing the workload contributes.
//
//   P1  `wfcnt_leaf` — the CALLEE, and `static` is the load-bearing field.
//                      `w-fence2` narrowed the inline fence to exempt a
//                      locally-defined callee only when its `.gl` defined record
//                      is **plain external** (`gl::plain_external_defined_names`
//                      — linkage byte `05`, flags `00`, at `/O1`). A `static`
//                      record reads linkage `03` (GRID-K), so the exemption does
//                      not reach it and `IlBundle::functions` refuses this whole
//                      TU at `locally-defined-callee` — that fence, alone.
//   P2  `wfcnt_use`  — the CALLER, whose one call names a symbol this TU defines.
//
// **AND BOTH BODIES ARE BYTE-EXACT AGAINST REAL c2, WHICH IS THE HALF THAT
// MAKES THIS A PRICE RATHER THAN A REFUSAL.** c2 inlines this callee — it is far
// under every measured ceiling — and the port reproduces the inlined caller
// through **mechanism I** (`c2_core::splice`, graded 723/723 exact when
// `w-inlfence` wrote its rung), so the per-function judge grades
// `fnbyte-exact 2` against a `fnbyte-denominator` of 2. The TU is refused all
// the same. That pair — *every emitted body byte-exact, obj emitted: none* — is
// exactly what the counter publishes, and `crates/c2-harness/tests/
// fence_count.rs` asserts each half separately so neither can drift.
//
// **WHAT THIS FIXTURE DOES NOT CLAIM.** It does not claim the TU would MATCH if
// the fence were lifted: a whole-obj verdict is a conjunction over the emit set,
// the sections and both tables, and per-function exactness is a statement about
// bodies (`docs/FUNCTION_BYTE_MATCH.md` §7). `fence-blocks-exact` counts TUs a
// named fence holds out of `match` whose bodies are already paid — a lower bound
// on that fence's cost, never a conversion forecast.
//
// **NOTHING FROM `docs/whitebox/` IS ADOPTED HERE.** No size bracket, no
// ceiling, no flag bit is read by this file or by the counter; the only fact it
// leans on is the categorical one already in `bind.rs` — a callee this TU
// defines — and the byte claim is checked against real c2's own obj rather than
// argued from a bracket.
//
// **`/O1` only, and it declares no profile** for the reason
// `wfence2_kept_local_callee.cpp` does not: the exemption's mode gate is `/O1`
// (board #1638), so at the default `/Ox` this TU is `NotImplemented`.

// P1 — the CALLEE. `static`, so the fence's exemption does not reach it.
static int wfcnt_leaf(int a) {
    return a + 1;
}

// P2 — the CALLER. One call to a name THIS TU defines; c2 inlines it and the
// port's mechanism-I splice reproduces the result byte for byte.
int wfcnt_use(int a) {
    return wfcnt_leaf(a);
}
