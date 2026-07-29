// **Negative** — an operand used twice. Every function here must keep refusing.
//
// This fixture exists because `return a + a;` was a **live mis-emit** in the
// straight-line integer class, and had been since that class was written. The port
// emitted `add r3,r3,r3`. c2 emits `rlwinm r3,r3,1,0,30` — `slwi r3,r3,1` — the
// same bytes it produces for `a * 2`:
//
//   int dbl(int a)  { return a + a; }   ->  5463083c   (slwi r3,r3,1)
//   int mul2(int a) { return a * 2; }   ->  5463083c
//
// A repeated leaf licenses c2's algebraic rewriter, and it takes the licence, so
// the operand stream stops being a faithful description of the instructions.
//
// It survived this long because no fixture used a parameter twice. Every existing
// positive is `a + b + c` or `a - b`, all distinct operands — and the FP leaf
// parser has had exactly this gate from the start, so the rule was known; it just
// was never applied on the integer side. That is the failure mode
// `fixtures/README.md` warns about: a green corpus is only as strong as its ability
// to separate the candidate rules, and this one had no separating case at all.
//
// `sub_self` and `mul_self` are here because the rewrite is not one rule — `a - a`
// is a constant zero and `a * a` has no shift form — so a gate written around the
// `+` case alone would still be guessing about the others. All three refuse.
//
// `via_assign` is the reason the gate cannot live only where the source text is
// read: nothing in that body repeats an operand. Substitution *creates* the
// repetition, so the check has to run on the resolved expression.
//
// `in_call_arg` covers the same stream reached through a call argument region.

int dbl(int a) { return a + a; }
int add_self_mid(int a, int b) { return a + b + a; }
int sub_self(int a) { return a - a; }
int mul_self(int a) { return a * a; }

int via_assign(int a) {
    int x = a;
    x = x + x;
    return x;
}

int g1(int);
int in_call_arg(int a) { return g1(a + a); }
