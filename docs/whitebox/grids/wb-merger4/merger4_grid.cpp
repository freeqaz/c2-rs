// merger4_grid.cpp — w-merger4's grid, frozen by CONTENT HASH before its first
// cl.exe (WB_MERGER4_PREREG_R2.md records the hash; w-keygen's rule that a
// hold-out frozen by NAME is not frozen).
//
// WHY IT EXISTS. Board #3103: with 0x10b3b167 (K1), 0x10b3b41b (K2) and
// 0x10b3b5fd (K3) ALL patched to `return 0`, `dk_join3` still collapses three
// copies of a common tail store to two. A fourth block merger exists, or the
// collapse has another cause.
//
// WHAT THE READ SAYS (WB_MERGER4_FINDINGS.md §2, addresses cited there):
// 0x10b36f7e — the pairwise tuple-equivalence test K1/K2/K3 use — has SEVEN
// callers, not three. Two of the four extra ones are non-DAG textual mergers
// reached from 0x10b3c2cc's LABEL class (tuple category 0x1b) under mode == 2,
// which #3103's candidate list does not name:
//
//   0x10b3baa8 -> 0x10b3a790   pairwise tail merge over a label's PREDECESSOR
//                              LIST; walks both predecessors backwards in
//                              lockstep through tuple+0x10, comparing with
//                              0x10b36f7e; commits with 0x10b36e93 /
//                              0x10bd417d / 0x10bd5952 / 0x10bd5648.
//                              NO 0x10b328da — no dependence DAG at all.
//   0x10b3ab86 -> 0x10b394f5   the same walk over PAIRS OF PREDECESSORS THAT
//                              BOTH END IN A CONDITIONAL BRANCH; commits with
//                              0x10b36e93 and creates a fresh label
//                              (0x10b9a455 / 0x10bd415e / 0x10bd3824).
//
// 0x10b3a790 carries a size budget `(-(DAT_10c2e310 != 0) & 0x12) + 2` — 2 at
// favor-SIZE, 0x14 = 20 at favor-SPEED — and a favor-size special case that
// rejects a ONE-tuple match when the predecessor's terminator is not a plain
// conditional branch. Families L and P below exist to put those on a cell.
//
// AXES CROSSED (values vary inside cells; #3102's rule):
//   N  number of duplicate arms         2 / 3 / 4
//   L  length of the common tail        1 / 2 / 4 statements
//   C  a call between the duplicated work and the join, and inside it
//   H  common code at the HEAD vs the TAIL of the arms
//   P  which arm carries the extra work
//   (crossed at run time with favor-size /O1 vs favor-speed /O1 /Ot, and /Od)
//
// METRIC: number of `stw` instructions naming dc_c in the function's PROC body
// of the /FAsc listing, and the whole-function instruction count.

extern int dc_a, dc_b, dc_c, dc_d, dc_e, dc_f, dc_g;
extern int dc_x0, dc_x1, dc_x2, dc_x3;
extern int dc_ext(int);

// ---------------------------------------------------------------- family N --
// The arm-COUNT axis. mg_arm3 is dk_join3 restated; it is the continuity cell
// with #3103 and must reproduce 1 copy on A0 / 2 copies on A123 at favor-size.

void mg_arm2(int c) {
    if (c) { dc_a  = 1; dc_c = 9; }
    else   { dc_b  = 2; dc_c = 9; }
}

void mg_arm3(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; }
    else        { dc_x0 = 3; dc_c = 9; }
}

void mg_arm4(int c, int d, int e) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; }
    else if (e) { dc_x0 = 3; dc_c = 9; }
    else        { dc_x1 = 4; dc_c = 9; }
}

// ---------------------------------------------------------------- family L --
// The common-tail LENGTH axis, arm count held at 3. 0x10b3a790's budget is 2 at
// favor-size and 20 at favor-speed, and its favor-size guard singles out a
// match of length exactly 1.

void mg_len1(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; }
    else        { dc_x0 = 3; dc_c = 9; }
}

void mg_len2(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; dc_d = 8; }
    else if (d) { dc_b  = 2; dc_c = 9; dc_d = 8; }
    else        { dc_x0 = 3; dc_c = 9; dc_d = 8; }
}

void mg_len4(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; dc_d = 8; dc_e = 7; dc_f = 6; }
    else if (d) { dc_b  = 2; dc_c = 9; dc_d = 8; dc_e = 7; dc_f = 6; }
    else        { dc_x0 = 3; dc_c = 9; dc_d = 8; dc_e = 7; dc_f = 6; }
}

// ---------------------------------------------------------------- family C --
// A call is an absolute scheduler barrier (#3069, 15/15), so any motion across
// one is provably not the scheduler's.

void mg_call3(int c, int d) {
    if (c)      { dc_a  = dc_ext(1); dc_c = 9; }
    else if (d) { dc_b  = dc_ext(2); dc_c = 9; }
    else        { dc_x0 = dc_ext(3); dc_c = 9; }
}

void mg_callin3(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = dc_ext(9); }
    else if (d) { dc_b  = 2; dc_c = dc_ext(9); }
    else        { dc_x0 = 3; dc_c = dc_ext(9); }
}

// ---------------------------------------------------------------- family H --
// Common code at the HEAD of each arm — the hoist direction (K2's shape) with
// three predecessors, which K2 does not cover.

void mg_head3(int c, int d) {
    if (c)      { dc_c = 9; dc_a  = 1; }
    else if (d) { dc_c = 9; dc_b  = 2; }
    else        { dc_c = 9; dc_x0 = 3; }
}

// ---------------------------------------------------------------- family P --
// WHICH arm carries the extra work. In mg_mid3 the middle arm's tail differs,
// so only the outer two can merge; in mg_cond2 the two predecessors of the join
// each end in a CONDITIONAL branch, which is 0x10b3ab86 / 0x10b394f5's shape
// and not 0x10b3baa8's.

void mg_mid3(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; dc_g = 5; }
    else        { dc_x0 = 3; dc_c = 9; }
}

void mg_cond2(int c, int d) {
    if (c) { if (d) { dc_a = 1; dc_c = 9; } else { dc_x2 = 6; } }
    else   { if (d) { dc_b = 2; dc_c = 9; } else { dc_x3 = 7; } }
}

// --------------------------------------------------------------- controls ---
// mg_none3 has NO common tail and must never merge in ANY image — it is the
// cell that catches a metric that counts something other than merging.
// mg_loop3 is dk_loop_join restated: #3103 records that it, too, merges under
// full K1/K2/K3 ablation.

void mg_none3(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 8; }
    else        { dc_x0 = 3; dc_c = 7; }
}

int mg_loop3(int n, int c) {
    int t = 0;
    for (int i = 0; i < n; ++i) {
        if (c) { dc_a = i; dc_c = 9; }
        else   { dc_b = i; dc_c = 9; }
        t += dc_e;
    }
    return t;
}
