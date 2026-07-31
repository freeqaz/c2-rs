// W4b2-i, corrected by W41: `return g(a) - 1` is the SAME framed body as the
// accepted `return g(a) + 1`, with a different immediate.
//
// This file spent from W4b2 to W41 in the *honest-rejection* lane, on this
// stated ground: "c2 does NOT canonicalize `-1` to `+(-1)`. Subtraction is
// non-commutative and off the verified 0x24-byte ADD frame." **That was an
// argument and not a capture, and it is false.** MEASURED
// (`work/w41/probe/p1.cpp`, `p5.cpp`, `/O1 /GS- /c`), the two bodies differ in
// exactly the immediate field of one word:
//
//   int f(int a){ return g(a) + 20; }   … bl ?g ; 38630014  addi r3,r3,20
//   int f(int a){ return g(a) - 20; }   … bl ?g ; 3863ffec  addi r3,r3,-20
//
// The refusal cost **0** free-function bodies on the 878-TU dc3 workload — which
// is why nothing ever contradicted it — and **3,559** *member*-call ones
// (`return p->m() - k;`, W41), because a container's `end() - 1` is written with
// a subtraction and a free function's `+ k` is not. A gate whose stated reason is
// an argument rather than a byte is `docs/GAPS.md` §6's recurring shape, and this
// one was found only because a different row needed the same locator.
//
// What genuinely is out of class is the neighbour it was grouped with: `* k`
// strength-reduces to a shift/add sequence and is not one `addi`
// (`w41_framed_member_call_neg.cpp`'s `n_mul`).
int g(int);
int f(int a) { return g(a) - 1; }
