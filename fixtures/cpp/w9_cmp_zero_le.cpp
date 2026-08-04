// W9 — the last ungraded emission production in `crates/c2-core/src/codegen/`.
//
// Found by the coverage sweep in `work/w-frame/sweep.py`, not by reading: build
// the harness with `-C instrument-coverage`, run it over ONLY the builds whose
// obj was byte-compared and matched (the 104 `Port=Match` fixtures at every one
// of the 12 `scripts/lanes.txt` flag lanes, plus the 8 matching workload TUs),
// and subtract that from the code reached at all. What is left is code the port
// will happily emit and the oracle has never seen.
//
// After correcting the graded profile twice, **one** emission production
// survived: `(Rel::Le, signed)` in `leaf/compare.rs` — the `a <= 0` zero fold —
// together with `encode_orc`, which nothing else in the crate calls.
//
//     neg   r11,a
//     orc   d,a,r11        <- the only `orc` the port can emit
//     srwi  r3,d,31
//
// Its comment reads *"/O1: as for `>` above — the `orc` consumes the dying
// `neg`"*, and that is exactly the hazard: the scratch register `d` is
// **11 at /O1 and 10 otherwise**, a rule with a witness at `>` and none of its
// own here. `wcf_neighbours.cpp::ctl_cmp` is `a > 0` and is the whole reason the
// `>` arm is graded; nothing anywhere was `a <= 0`.
//
// `w6_rel_k.cpp` looks like it should have caught this and cannot: every one of
// its twenty bodies compares against a NON-zero literal (5, -3, 1, 32767,
// -32768), so it drives the general relational spines and never the zero folds.
// A fixture family can be thorough on the axis it varies and blind on the one it
// holds fixed.
//
// The other three zero folds are here as controls rather than as new cells: two
// are already graded and one is the mode-identical sibling, so if `le_s` moved
// and they did not, the defect is in the `<=` arm and not in the spine.
int le_s(int a) { return a <= 0; }   // THE ungraded cell: neg / orc / srwi31
int gt_s(int a) { return a > 0; }    // graded control: neg / andc / srwi31
int le_u(unsigned a) { return a <= 0u; } // folds to `a == 0`: cntlzw / rlwinm
int lt_s(int a) { return a < 0; }    // folds to the bare sign bit: srwi31

// The three UNSIGNED zero folds the same sweep found beside `le_s`. Two of them
// are constant folds — `a < 0u` is always false and `a >= 0u` always true — and
// a constant fold is exactly the kind of arm that looks too obvious to grade and
// is therefore never graded. With these, all twelve cells of the comparison
// zero-fold table have a byte behind them.
int gt_u(unsigned a) { return a > 0u; }  // folds to `a != 0`: addic / subfe
int lt_u(unsigned a) { return a < 0u; }  // constant false: li r3,0
int ge_u(unsigned a) { return a >= 0u; } // constant true:  li r3,1
