// dagclients_grid2.cpp — w-dagclients' SECOND grid, frozen by content hash
// before its first cl.exe. It is a NEW cell set, not an edit of
// dagclients_grid.cpp (whose hash and results stand unchanged); R2 §1's rule.
//
// WHY IT EXISTS. Grid 1's ablation (A3 = K3 `0x10b3b5fd` patched to
// `xor eax,eax; ret 0xc`) produced a byte-IDENTICAL obj at all five S-OPT
// levels. That is an ABSENCE and this repo does not bank those: the honest
// reading is "grid 1 never reached K3", not "K3 does not reorder". This grid
// is the attempt to reach it.
//
// WHAT K3 NEEDS, from the read. K3 is entered from `0x10b3c2cc` on the branch
// class K1 does NOT take (`tuple+0x34 != 0` or opcode in {0x2e4,0x21,0x22}),
// only after K2 `0x10b3b41b` has returned 0, and only when `DAT_10c2e310 == 0`
// = **favor-SIZE** (`/O1`, `/O2 /Os`; board #1611). Unlike K1 and K2 it does
// not take the branch's own target as the second block: it calls
// `FUN_10b35f88` @ `0x10b35f88` to SEARCH, and failing that walks the label's
// predecessor list `label[10]` for another `0x12` tuple targeting the same
// label. So its shape is **two or more predecessors of one join point that are
// not the two arms of a single `if`** — a three-way chain, or a switch.
//
// REGISTERED BEFORE THE FIRST RUN OF THIS FILE (scored in the findings doc):
//   G2-1  A3 (K3 ablated) changes at least one function of this grid at
//         favor-SIZE (/O1 or /O2 /Os)                                p = 0.45
//   G2-2  A3 changes NOTHING at favor-SPEED (/O1 /Ot, /O2) — K3's own
//         gate is `DAT_10c2e310 == 0`                                p = 0.85
//   G2-3  A123 differs from A0 on at least one function here         p = 0.90
//   G2-4  dk_join3 emits ONE copy of the dc_c store at favor-size on the
//         real image (a three-predecessor join is merged at all)      p = 0.60
// A3 == A0 everywhere here is reported as "K3 STILL NOT REACHED", never as
// "K3 does not reorder".

extern int dc_a, dc_b, dc_c, dc_d, dc_e;
extern int dc_f0, dc_f1, dc_f2;
extern int dc_ext(int);

// Three predecessors of one join, each ending in the same statement.
void dk_join3(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; }
    else        { dc_f0 = 3; dc_c = 9; }
}

// Four predecessors of one join.
void dk_join4(int c, int d, int e) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_b  = 2; dc_c = 9; }
    else if (e) { dc_f0 = 3; dc_c = 9; }
    else        { dc_f1 = 4; dc_c = 9; }
}

// Three predecessors, common statement NOT in the same position in each.
void dk_join3s(int c, int d) {
    if (c)      { dc_a  = 1; dc_c = 9; }
    else if (d) { dc_c  = 9; dc_b = 2; }
    else        { dc_f0 = 3; dc_c = 9; }
}

// A switch: several predecessors of the join reached through a jump table or a
// compare chain, which is the `tuple+0x34 != 0` branch class by construction.
void dk_switch(int c) {
    switch (c) {
    case 0:  dc_a  = 1; dc_c = 9; break;
    case 1:  dc_b  = 2; dc_c = 9; break;
    case 2:  dc_f0 = 3; dc_c = 9; break;
    case 3:  dc_f1 = 4; dc_c = 9; break;
    default: dc_f2 = 5; dc_c = 9; break;
    }
}

// A join whose predecessors end in a CALL plus the common statement — a call is
// a region ender for the scheduler (0x14), so any merge here is provably not
// the scheduler's doing.
void dk_call_join(int c, int d) {
    if (c)      { dc_a = dc_ext(1); dc_c = 9; }
    else if (d) { dc_b = dc_ext(2); dc_c = 9; }
    else        { dc_d = dc_ext(3); dc_c = 9; }
}

// A loop whose body has two predecessors of the latch — the label class
// (0x1b) that K3's predecessor walk keys on.
int dk_loop_join(int n, int c) {
    int t = 0;
    for (int i = 0; i < n; ++i) {
        if (c) { dc_a = i; dc_c = 9; }
        else   { dc_b = i; dc_c = 9; }
        t += dc_e;
    }
    return t;
}
