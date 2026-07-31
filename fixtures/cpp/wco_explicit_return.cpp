// WCO — the ALARM: **a lone statement call followed by an explicit `return;`**
// is a TAIL call, and the port was emitting a framed body for it.
//
// `void h1() { g1(); return; }` is `b ?g1@@YAXXZ` — four bytes, the same as
// `void h1() { g1(); }`. c2 records the fallthrough as a SECOND `3A <label>`
// branch to the same label the return plumbing then uses and emits nothing for
// it, so the two objs are byte-identical.
//
// The port emitted the 36-byte framed Class A body instead: the statement-call
// production's tail-call probe runs the plumbing at `BODY_SCOPE_DEPTH`, which
// does not parse the double `3A`, so the body fell through to
// `parse_call_sequence` — where a `debug_assert` said this state was
// unreachable and was wrong. MEASURED: `Port=Mismatch @ offset 2` against the
// reference obj, and the wrong bytes were **live on mainline**.
//
// A `debug_assert` compiles out of the release scan and 98.5 % of the workload
// is `vocab-gap` and never byte-compared, so nothing in the lane saw it. It
// surfaced only because a debug `c2rs census` on a real workload TU
// (`src/system/hamobj/DancerSequence.cpp`) tripped the assertion.
//
// Every function here must be in class: `c2rs census` N/N.

struct O {
    void v();
    void va(int);
    int gi();
};

void g1();
void g2(int);
void g3(int, int);
void g4(int, int, int);
int gi1();
int gi2(int);

// ---- the row: ONE call, discarded, then an explicit `return;` ---------------
// Each is the bare tail branch, identical to the same body without the
// `return;`. The argument count walks the whole marshalling ladder, because the
// wrong path emitted a frame for every one of them.
void h_none() { g1(); return; }
void h_one(int a) { g2(a); return; }
void h_two(int a, int b) { g3(a, b); return; }
void h_swap(int a, int b) { g3(b, a); return; }
void h_three(int a, int b, int c) { g4(a, b, c); return; }
void h_rot(int a, int b, int c) { g4(c, a, b); return; }
// …with an argument that needs the leaf selector rather than a bare move.
void h_setup(int a) { g2(a + 1); return; }
// …and a literal.
void h_lit() { g2(7); return; }

// ---- the same, with the result DISCARDED from a non-void callee -------------
void h_discard() { gi1(); return; }
void h_discard_arg(int a) { gi2(a); return; }

// ---- TWO or more calls keeps the frame --------------------------------------
// The boundary in the other direction: these really are Class A framed
// sequences, and the fix must not have moved them.
void h2_calls() { g1(); g1(); return; }
void h3_calls(int a) { g2(a); g1(); g1(); return; }

// ---- a value tail is not this row either ------------------------------------
// One call and `return <literal>;` is the framed `bl` + `li r3,k`.
int h_lit_tail() { g1(); return 5; }
