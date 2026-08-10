// w-frame783 — w-selbind's OVER-EMIT counterexample, re-run under the shipped
// frame relaxation (board #2820, `Bindings::selective` clause 4).
//
// `.ex` splits into THREE segments; `.gl` carries framed records for `u` and
// `f` and ZERO unclaimed runs of either kind, so clause 3 is satisfied; and
// c2's 833-byte obj holds ONE 8-byte `.text` — `?f@@YAHH@Z` alone. Binding the
// record set emits `u` as well: `Port=Mismatch @ offset 8`.
//
// A frame relaxation widens what the walk can SEE, so this cell has to be
// re-graded, not inherited. The two ways it could go wrong are different:
//   * the walk binds MORE records and the count reaches 3 == segments, which
//     routes the TU through the 1:1 arm and out of clause 4 entirely;
//   * clause 4 stops firing for any other reason.
// Either shows up as `Port=Mismatch` on this file.
inline int u(int a) { return a + 2; }
inline int v(int a) { return u(a) + 3; }
int f(int a) { return a + 1; }
