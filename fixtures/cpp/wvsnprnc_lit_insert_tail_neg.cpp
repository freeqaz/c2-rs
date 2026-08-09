// **W-VSNPRNC — the FENCE around the spliced literal.** The TU must come out
// `NotImplemented`, and six of its seven cells are shapes a rule fitted to
// `wvsnprnc_lit_insert_tail.cpp` would happily emit wrong bytes for.
//
// **`n2` is the seventh and it is IN CLASS**, deliberately: it is the shipped
// WLB two-slot cell, which this lane did not touch. It is here as the
// DISJOINTNESS control — the census must keep reading it `ok` through the
// shipped path while the six around it refuse, which is the executable form of
// "the splice class and `one_moved_at_two` do not overlap". Its presence does
// not weaken the fixture: one refused function is enough to keep the whole TU
// `NotImplemented`, and the census line is the assertion.
//
// The widening admits *the formals in order with one constant inserted*. What
// makes that a class and not a guess is that it requires **every formal, once,
// in source order**. Each `n` below breaks exactly one clause of that sentence.
//
// **`n0` is the important one.** It is the WLB fence's own measured
// counterexample, `g3(c,a,7)`, which c2 lowers as
// `mr r11,r5 ; mr r4,r3 ; li r5,7 ; mr r3,r11` — the `li` *inside* the break
// walk, through the scratch register. Any rule fitted to the positive fixture's
// four cells mis-emits it. It was refused before this lane and it must still be.
//
// STRUCTURAL BLIND SPOT: like its positive twin, every cell is `int` formals, a
// tail call and an external callee, so this file fences the *slot list* and
// nothing else. A wrong lowering that depended on the formal TYPE or on the call
// being framed would pass every cell here.

int neg_target_3(int, int, int);
int neg_target_2(int, int);
int neg_target_4(int, int, int, int);
int neg_target_9(int, int, int, int, int, int, int, int, int);

// n0 — **`g3(c,a,7)`**: one formal moves UP while another moves DOWN. A real
// 2-cycle beside a literal, which no splice produces. c2 breaks it through r11
// and puts the `li` in the middle of the walk.
int n0(int a, int b, int c) {
    return neg_target_3(c, a, 7);
}

// n1 — two formals TRANSPOSED beside a literal. The list is a permutation, not
// an insertion: `b` and `a` are out of source order.
int n1(int a, int b) {
    return neg_target_3(b, a, 7);
}

// n2 — a formal DROPPED: `a` never reaches the callee, so the formals are in
// order but not exhaustive. **This is the shipped WLB cell `g2(c,7)` and it is
// IN CLASS** — `lit_insert_at` returns `None` for it and `one_moved_at_two`
// takes it, which is exactly the disjointness this file asserts. If a future
// widening of the splice class ever swallowed this list, the census line for
// this cell would not change but the code path would; the `#[test]`
// `a_dropped_formal_is_not_an_insertion` is what pins the path.
int n2(int a, int b, int c) {
    return neg_target_2(c, 7);
}

// n3 — TWO literals beside a move. One `li` is a schedule this grid measured;
// two of them beside a move is a different question and is refused rather than
// assumed to be the same one.
int n3(int a, int b) {
    return neg_target_4(a, 5, b, 6);
}

// n4 — an inserted literal that does NOT fit `li`'s signed 16-bit immediate.
// The shape is right and the constant is not, which is the direction a fence
// usually misses.
int n4(int a, int b) {
    return neg_target_3(a, 70000, b);
}

// n5 — a splice past the eight argument registers: nine slots, so the last is
// stack-homed and the whole marshalling is a different shape.
int n5(int a, int b, int c, int d, int e, int f, int g, int h) {
    return neg_target_9(a, b, c, d, 0, e, f, g, h);
}

// n6 — a formal REPEATED. Every slot below the literal names the formal at its
// own index, so the list looks like an insertion until the duplicate is read;
// c2 emits a dead move for the repeat.
int n6(int a, int b) {
    return neg_target_3(a, 0, a);
}
