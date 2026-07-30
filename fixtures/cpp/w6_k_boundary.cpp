// **Negative** — `== ` and `!=` against `i16::MIN`, which must refuse.
//
// Both spines start by forming `a - k` as `addi r11, a, -k`. That needs `-k` to fit
// the signed 16-bit immediate, and for `k == -32768` it does not: negating it
// overflows. The port emitted a wrong immediate.
//
// The interesting part is why `w6_rel_k.cpp` did not catch it despite deliberately
// testing both i16 boundaries. It tests `a <= -32768`, and `<=` reaches an entirely
// different spine — `li r10,-1 ; srwi ; srawi ; subfc ; adde` — which never negates
// `k` and is perfectly happy at the boundary. So the fixture *did* probe the
// boundary, and *did* probe every relation, but not the boundary and the vulnerable
// relations together. A generated sweep over (relation x k) found it immediately.
//
// That is the same lesson as `il_reassoc.cpp` in a subtler form: covering each axis
// separately is not covering the cross product, and a hand-written corpus tends to
// vary one axis at a time because that is how a person reasons about coverage.
//
// `ne_min` and `eq_min` refuse; the `<=`/`<`/`>=`/`>` forms at the same `k` are in
// `w6_rel_k.cpp` and must keep emitting, so this file and that one together pin
// which spines actually depend on negating the literal.

int eq_min(int a) { return a == -32768; }
int ne_min(int a) { return a != -32768; }
