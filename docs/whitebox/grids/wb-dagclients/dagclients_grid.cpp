// dagclients_grid.cpp — w-dagclients' obj grid for board #3071: do the four
// DAG-builder clients that BYPASS the region finder reorder tuples?
//
// FROZEN BEFORE THE FIRST cl.exe OF THIS LANE, BY CONTENT HASH. Predictions are
// in docs/whitebox/WB_DAGCLIENTS_PREREG_R2.md, committed in the same commit as
// this file. R1 (docs/whitebox/WB_DAGCLIENTS_PREREG.md) was committed at
// cff4a8db before the first grep of the Ghidra export.
//
// Compile with the real cl.exe 16.00.11886.00 under wibo. The S-OPT axis is
// #1611's deciding quad plus /Od:
//     /Od          /O1          /O1 /Ot          /O2 /Os          /O2
// (favor-size = /O1 and /O2 /Os; favor-speed = /O1 /Ot, /O2, /Ox — established
// black-box by wb-memcpy over 180 cells, board #1611.)
//
// THE RED-CAPABLE CELL IS dt_sink. If none of K1/K2/K3 reorders tuples, it must
// look exactly like dt_none: two copies of the dc_c store, because dc_c = 9 is
// NOT a common suffix of the two arms as written. One copy means a tuple moved.
//
// dt_mid / dt_far are the ATTRIBUTION cells: FUN_10b397ba @ 0x10b397ba bounds
// K1's backward scan at 0x1d = 29 tuples. A merge in dt_sink and dt_mid but NOT
// in dt_far exhibits that window in emitted code, which a generic store-sinker
// would have no reason to respect.

extern int dc_a, dc_b, dc_c;
extern int dc_f0,  dc_f1,  dc_f2,  dc_f3,  dc_f4,  dc_f5,  dc_f6,  dc_f7,
           dc_f8,  dc_f9,  dc_f10, dc_f11, dc_f12, dc_f13, dc_f14, dc_f15,
           dc_f16, dc_f17, dc_f18, dc_f19, dc_f20, dc_f21, dc_f22, dc_f23,
           dc_f24, dc_f25, dc_f26, dc_f27, dc_f28, dc_f29;
extern int dc_sink_ext(int);

// ---------------------------------------------------------------------------
// Family T — tail merge / cross-jump (K1 = 0x10b3b167, K3 = 0x10b3b5fd).
// Common work at the END of the two arms; the sink helper is 0x10b3ada1
// (nodes whose DAG fanout is 0) and the splice is 0x10bd38b0 (insert BEFORE
// the branch).
// ---------------------------------------------------------------------------

// Common statement is already the SUFFIX of both arms: a naive textual suffix
// matcher merges this. Baseline, not discriminating.
void dt_sfx(int c) {
    if (c) { dc_a = 1; dc_c = 9; }
    else   { dc_b = 2; dc_c = 9; }
}

// THE RED-CAPABLE CELL. Common statement is NOT the suffix of arm B, and dc_b
// is a distinct global, so sinking dc_c = 9 past it is dependence-legal.
void dt_sink(int c) {
    if (c) { dc_a = 1; dc_c = 9; }
    else   { dc_c = 9; dc_b = 2; }
}

// Same shape, but the tail store is through an unknown pointer and may alias
// dc_c, so the sink is dependence-ILLEGAL. Discriminates "DAG-gated motion"
// from "motion regardless of dependence".
void dt_dep(int c, int *p) {
    if (c) { dc_a = 1; dc_c = 9; }
    else   { dc_c = 9; *p = 2;   }
}

// Negative control: nothing is common, so nothing may merge at any level.
void dt_none(int c) {
    if (c) { dc_a = 1; }
    else   { dc_b = 2; }
}

// Attribution, inside K1's 29-tuple backward window: ~10 intervening stores.
void dt_mid(int c) {
    if (c) { dc_a = 1; dc_c = 9; }
    else   { dc_c = 9;
             dc_f0 = 0; dc_f1 = 1; dc_f2 = 2; dc_f3 = 3; dc_f4 = 4;
             dc_f5 = 5; dc_f6 = 6; dc_f7 = 7; dc_f8 = 8; dc_f9 = 9; }
}

// Attribution, OUTSIDE K1's 29-tuple backward window: 30 intervening stores,
// each of which lowers to more than one tuple.
void dt_far(int c) {
    if (c) { dc_a = 1; dc_c = 9; }
    else   { dc_c = 9;
             dc_f0  = 0;  dc_f1  = 1;  dc_f2  = 2;  dc_f3  = 3;  dc_f4  = 4;
             dc_f5  = 5;  dc_f6  = 6;  dc_f7  = 7;  dc_f8  = 8;  dc_f9  = 9;
             dc_f10 = 10; dc_f11 = 11; dc_f12 = 12; dc_f13 = 13; dc_f14 = 14;
             dc_f15 = 15; dc_f16 = 16; dc_f17 = 17; dc_f18 = 18; dc_f19 = 19;
             dc_f20 = 20; dc_f21 = 21; dc_f22 = 22; dc_f23 = 23; dc_f24 = 24;
             dc_f25 = 25; dc_f26 = 26; dc_f27 = 27; dc_f28 = 28; dc_f29 = 29; }
}

// ---------------------------------------------------------------------------
// Family H — head merge / hoist (K2 = 0x10b3b41b). Common work at the START of
// the two arms; the hoist helper is 0x10b3ad62 (nodes whose DAG pred count is
// 0) and the splice is 0x10bd3892 (insert AFTER the branch).
// ---------------------------------------------------------------------------

void dh_pfx(int c) {
    if (c) { dc_c = 9; dc_a = 1; }
    else   { dc_c = 9; dc_b = 2; }
}

void dh_hoist(int c) {
    if (c) { dc_c = 9; dc_a = 1; }
    else   { dc_b = 2; dc_c = 9; }
}

void dh_dep(int c, int *p) {
    if (c) { dc_c = 9; dc_a = 1; }
    else   { *p = 2;   dc_c = 9; }
}

// ---------------------------------------------------------------------------
// K4 — /QXSTALLS (0x10c1ce93, under the listing writer 0x10b71d8f). It builds
// the dependence DAG over the WHOLE function in one call, bypassing both
// FUN_10be5d4b's region enders and its 0x50-tuple cap, so the shapes that
// matter are: an interior barrier of each kind, and a body past the cap.
// Every cell is compiled twice, /QXSTALLS off and on, and byte-compared.
// ---------------------------------------------------------------------------

// No interior barrier — the shape wb-dagorder's grid already covered.
int dq_plain(int x) {
    return dc_a + x * 3 + dc_b + dc_c;
}

// Interior conditional branch (region ender 0x12).
int dq_branch(int x) {
    int t = dc_a + x;
    if (x > 3) { t += dc_b * 2; } else { t -= dc_c; }
    return t + dc_a;
}

// Interior call (region ender 0x14).
int dq_call(int x) {
    int t = dc_a + x;
    t += dc_sink_ext(t);
    return t + dc_b * 3;
}

// Interior label (region ender 0x1b) — a loop.
int dq_loop(int x) {
    int t = 0;
    for (int i = 0; i < x; ++i) { t += dc_a * i + dc_b; }
    return t + dc_c;
}

// Past the 0x50-tuple region cap in a single straight-line body.
void dq_big(void) {
    dc_f0  = 0;  dc_f1  = 1;  dc_f2  = 2;  dc_f3  = 3;  dc_f4  = 4;
    dc_f5  = 5;  dc_f6  = 6;  dc_f7  = 7;  dc_f8  = 8;  dc_f9  = 9;
    dc_f10 = 10; dc_f11 = 11; dc_f12 = 12; dc_f13 = 13; dc_f14 = 14;
    dc_f15 = 15; dc_f16 = 16; dc_f17 = 17; dc_f18 = 18; dc_f19 = 19;
    dc_f20 = 20; dc_f21 = 21; dc_f22 = 22; dc_f23 = 23; dc_f24 = 24;
    dc_f25 = 25; dc_f26 = 26; dc_f27 = 27; dc_f28 = 28; dc_f29 = 29;
}
