// **Negative** — additive chains deeper than the enumerated region. All must refuse.
//
// Found by a Fable architecture review, which observed that the acceptance region of
// the canonicalization rule was *larger* than the region the sweep enumerates: the
// sweep goes to four leaves, `canonicalize_chain` accepted any length, and the
// additive path has no depth backstop at all — its accumulator is r11 forever, where
// the multiplicative path is at least bounded by codegen's r9 scratch floor.
//
// That prediction was correct. Probing 5- to 8-leaf chains found 10 mis-emits, all of
// the same shape — **three or more register subtractions followed by an addition**:
//
//   a - b - c - d       byte-exact
//   a - b - c + d       byte-exact
//   a - b - c - d + e   MIS-EMIT
//
// So c2's intermediate allocation diverges again at a depth that four leaves cannot
// reach, and the port was emitting on extrapolation from short chains. Exactly the
// shape of the per-chain accumulator bug (`il_accum4.cpp`), one level further out —
// and it survived a fully green 11,664-case 4-leaf sweep, because a bound that is
// wrong only past the enumerated edge is invisible from inside it.
//
// The invariant this fixture defends, now written into the parser and GAPS:
// **the acceptance region of a rewrite rule must be a subset of the region actually
// enumerated.** Raising the bound requires extending the sweep first, not the other
// way round.
//
// Pure additions and pure multiplications are deliberately *not* bounded — five-leaf
// forms of both are verified byte-exact — so `add5` and `mul5` are the separating
// cases. A gate that simply refused all deep chains would refuse those too.

int sub3_add1(int a, int b, int c, int d, int e) { return a - b - c - d + e; }
int sub4_add2(int a, int b, int c, int d, int e, int f) { return a - b - c - d - e + f; }
int sub3_add3(int a, int b, int c, int d, int e, int f, int g) { return a - b - c - d + e + f + g; }
