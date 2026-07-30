// Four-leaf additive chains — the shortest shape that can separate c2's two
// intermediate-register rules. All must emit, byte-exact.
//
// c2 decides accumulator-versus-descending **once for the whole chain**, not per
// operation. If the chain contains any addition, every intermediate reuses r11 —
// *including the subtractions ahead of that addition*. Only a chain with no addition
// at all gives each intermediate its own descending register:
//
//   a - b - c - d   ->  subf r11,r4,r3 ; subf r10,r5,r11 ; subf r3,r6,r10
//                       descending: r11, r10, then r3
//   a + b - c - d   ->  subf r11,r5,r3 ; subf r11,r6,r11 ; add r3,r11,r4
//                       r11 twice, even though both of those are subtractions
//   a + b + c - d   ->  subf r11,r6,r3 ; add r11,r11,r4 ; add r3,r11,r5
//   a - b - c + d   ->  subf r11,r4,r3 ; subf r11,r5,r11 ; add r3,r11,r6
//
// The port decided per operation — collapse on an `add`, descend on a `subf` — which
// is indistinguishable from the real rule at **one** intermediate. Every three-leaf
// chain has one intermediate. So `a - b - c` and `a + b - c` were byte-exact and the
// bug was invisible until four leaves gave a chain two intermediates with mixed
// operators. A generated sweep over 11,664 four-leaf chains found 270 of them.
//
// This is the same trap as `w5_chain.cpp`, one leaf deeper: that fixture caught the
// port reusing one scratch where c2 descends, and the note in `fixtures/README.md`
// about adding "the neighbour that would look the same under a plausible wrong rule"
// came from it. The neighbour needed here is not another operator mix — it is another
// *length*, because the rules only diverge once there are two intermediates.
//
// The reassociation these also undergo is in `il_reassoc.cpp`; here every chain is
// already written in canonical order so the two effects stay separable.

int sub4(int a, int b, int c, int d) { return a - b - c - d; }
int add_sub_sub(int a, int b, int c, int d) { return a + b - c - d; }
int add_add_sub(int a, int b, int c, int d) { return a + b + c - d; }
int sub_sub_add(int a, int b, int c, int d) { return a - b - c + d; }
int add4(int a, int b, int c, int d) { return a + b + c + d; }
int mul4(int a, int b, int c, int d) { return a * b * c * d; }
int sub3(int a, int b, int c) { return a - b - c; }
int add_sub3(int a, int b, int c) { return a + b - c; }
int sub_lit_tail(int a, int b, int c) { return a - b - c - 1; }
