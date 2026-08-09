// w-ifn — BLOCK ORDER for `cflow-if-2` and `cflow-if-n`.
//
// `wb-loop` (#1900–#1907) closed block order for the LOOP class: decision-tree
// switches emit arms in REVERSE source order, jump-table switches in source
// order, everything else source order, and a block reachable only as a loop
// exit is SUNK past the function's return.  The commission says `if-2`/`if-n`
// is exactly the shape that was NOT verified, so nothing below is inherited.
//
// Every cell is framed (it calls `gx`), so the epilogue is a real materialised
// block and the question "where did each arm land" has an answer in the bytes.

extern "C" int gx(int);
extern "C" int gy(int);
extern "C" void gv(int);

// ---- B1: the mmio shape — two guards, each `return K`, then the body.
extern "C" int b1(int a, int b) {
    if (a == 0) return 5;
    if (b == 0) return 11;
    gx(a);
    return 0;
}

// ---- B2: an if/ELSE with a join.  Which arm is laid down first?
extern "C" int b2(int a) {
    if (a == 0) { gx(1); } else { gy(2); }
    return 7;
}

// ---- B3: three guards — `cflow-if-n` with one more arm than mmio.
extern "C" int b3(int a, int b, int c) {
    if (a == 0) return 5;
    if (b == 0) return 11;
    if (c == 0) return 13;
    gx(a);
    return 0;
}

// ---- B4: the guard arm is NOT a return — it is a call, so the arm has to be
//          a real block that falls through to the join.
extern "C" int b4(int a) {
    if (a == 0) { gx(1); }
    gy(2);
    return 0;
}

// ---- B5: a NESTED if.  Does the inner arm land inside the outer one?
extern "C" int b5(int a, int b) {
    if (a == 0) {
        if (b == 0) return 5;
        return 11;
    }
    gx(a);
    return 0;
}

// ---- B6: the guard's arm is BIG.  A sinking rule would move it past the
//          return; a source-order rule leaves it where it is written.
extern "C" int b6(int a, int b) {
    if (a == 0) { gx(1); gy(2); gx(3); gy(4); return 5; }
    gv(b);
    return 0;
}

// ---- B7: the SAME body with the guard's sense inverted, so the arm and the
//          fallthrough swap in the source.  Separates "source order" from
//          "the taken arm goes last".
extern "C" int b7(int a, int b) {
    if (a != 0) { gv(b); return 0; }
    gx(1); gy(2); gx(3); gy(4);
    return 5;
}

// ---- B8: two guards that share ONE arm through `||`.
extern "C" int b8(int a, int b) {
    if (a == 0 || b == 0) return 5;
    gx(a);
    return 0;
}

// ---- B9: an early return in the MIDDLE of a straight run, mmioClose's shape:
//          call, test the result, return it, continue.
extern "C" int b9(int a) {
    int r = gx(a);
    if (r != 0) return r;
    int s = gy(a);
    if (s != 0) return s;
    gv(a);
    return 0;
}
