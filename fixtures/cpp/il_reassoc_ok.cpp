// The **positive** side of the reassociation boundary — chains whose source order
// already is c2's canonical order, and which must keep emitting.
//
// These exist so the two gates in `il_reassoc.cpp` cannot be "fixed" by refusing
// every mixed `+`/`-` chain or every chain containing a subtraction. c2 applies the
// negative terms of an additive chain first, so a chain whose subtractions already
// precede its additions needs no rewriting and is byte-exact as written:
//
//   a - b + c   ->  subf r11,r4,r3 ; add r3,r11,r5
//   a - b - c   ->  subf r11,r4,r3 ; subf r3,r5,r11
//
// And a subtraction of a *literal* never emits an instruction at all — it folds
// into the running `addi` immediate — so it cannot be out of order:
//
//   a + b - 1   ->  add r11,r3,r4 ; addi r3,r11,-1
//
// Pair each of these with its neighbour in `il_reassoc.cpp`: `a - b + c` here
// against `a + b - c` there differ by nothing but which operator comes first, and
// one is byte-exact while the other is a mis-emit. That is the whole content of the
// rule, and only both files together express it.

int sub_then_add(int a, int b, int c) { return a - b + c; }
int sub_then_sub(int a, int b, int c) { return a - b - c; }
int add_then_lit_sub(int a, int b) { return a + b - 1; }
int asc_add3(int a, int b, int c) { return a + b + c; }
int asc_mul3(int a, int b, int c) { return a * b * c; }
int asc_add2(int a, int b) { return a + b; }
int asc_sub2(int a, int b) { return a - b; }
int lit_only(int a) { return a + 1 - 2; }
