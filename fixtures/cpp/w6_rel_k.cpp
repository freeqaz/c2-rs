// W6 — every relation against a **non-zero** literal, signed and unsigned.
//
// The `k == 0` folds and the two `>` spines landed earlier; `<`, `>=` and `<=`
// were held out because the characterization could not pin their instruction
// *order* from reasoning alone. Live captures settled it, and the answers are
// not symmetric in the way a guess would assume:
//
//   * signed `<` is the signed `>` spine with two operands swapped and nothing
//     else changed — same six instructions, same register numbers. One of the
//     swapped words is an `eqv`, which is commutative, so the swap is invisible
//     in the value and shows up only in the bytes;
//   * unsigned `<` needs `k` in a register (the borrow it wants is the one out of
//     `a - k`, and `subfic` only computes `SIMM - rA`), so it is four
//     instructions where unsigned `>` is three — and that shifts every later
//     register down one;
//   * signed `>=`/`<=` sum two sign terms and a borrow with a single `adde`. One
//     term is `srawi …,31` (0/−1), the other `rlwinm …,1,31,31` (0/1); they are
//     NOT interchangeable, and which operand gets which flips between `>=` and
//     `<=`. The two shifts are emitted in *source* order, so `<=` — whose left
//     operand is the literal — emits them in the opposite order from `>=`;
//   * unsigned `<=` is the only shape whose literal rides in a `subfic`
//     immediate, so it is three instructions, and its `li r10,-1` is emitted
//     *before* the `subfic` even though it takes the lower register number
//     (emission order is not register-number order).
//
// The `k` values are deliberately spread: small, 1, negative, and both i16
// boundaries. A spine hardcoded around a single `k` would still match `5`.

int slt(int a) { return a < 5; }
int sge(int a) { return a >= 5; }
int sle(int a) { return a <= 5; }
int sgt(int a) { return a > 5; }

int ult(unsigned a) { return a < 5u; }
int uge(unsigned a) { return a >= 5u; }
int ule(unsigned a) { return a <= 5u; }
int ugt(unsigned a) { return a > 5u; }

int nlt(int a) { return a < -3; }
int nge(int a) { return a >= -3; }
int nle(int a) { return a <= -3; }
int ngt(int a) { return a > -3; }

int one_lt(int a) { return a < 1; }
int one_ule(unsigned a) { return a <= 1u; }
int one_uge(unsigned a) { return a >= 1u; }

int hi_ge(int a) { return a >= 32767; }
int lo_le(int a) { return a <= -32768; }
int hi_ult(unsigned a) { return a < 32767u; }
int hi_ule(unsigned a) { return a <= 32767u; }
