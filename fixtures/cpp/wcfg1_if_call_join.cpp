// **W-CFG1 — the two-armed `if`/`else` whose arms are CALLS.** The port's first
// `cflow-if-n` body and its first intra-section `b` to a JOIN block.
//
// `src/system/negate_test.cpp` is `p0`/`p1` verbatim and it is a FRONTIER TU:
// its two emitted functions are this shape, they are byte-identical to one
// another, and it converts on this class or on none. This file is the class's
// FENCE — the workload contains exactly one instance and one instance cannot
// tell a rule from a coincidence (board #260).
//
//     mflr/stw/stwu -96
//     mr    r10,r3         the park: the scrutinee straddles two `bl`s
//     mr    r3,r4          the hoist: both arms share one argument, so its
//                          setup is in the ENTRY block, above every branch
//     li    r11,0          the result home
//     cmpwi cr6,r10,K1     ONE compare, read at LT and again at EQ
//     bt    24,$LN1
//     bt    26,$LN1
//     cmpwi cr6,r10,K2
//     bt    24,$LN2
//     bl    <hi>
//     b     $LN8           the join
//     bl    <lo>
//     $LN8: mr r11,r3
//     $LN1: mr r3,r11      -- and this pair is NOT removable; see the emitter
//     addi/lwz/mtlr/blr
//
// The STRUCTURAL axes are held by `wcfg1_if_call_join_neg.cpp`; what varies here
// is the VALUE axis, plus the one structural variation that must NOT change a
// byte. Board #198's rule applied in both directions.
//
// ---- p1 is the separating cell ---------------------------------------------
//
// `p0` and `p1` differ in source and must not differ in `.text`: c2 deletes the
// empty middle arm and emits one branch for both `b == K1` and `!(b != K1)`,
// with the sense inverted once. Nothing else in this corpus grades two spellings
// that must produce one word, and it is exactly what the reader's `1F`/`20`
// alternation claims. If the two ever diverge, the alternation is wrong and the
// spelling the workload does not contain has to be refused.

struct Node;
extern const Node *hi(void *, float);
extern const Node *lo(void *, float);
extern int *ihi(void *, float);
extern int *ilo(void *, float);

enum Blend { b0 = 0, b1 = 1, b2 = 2 };

// **Every arm is braced, and that is load-bearing.** `54 <k>` carries the scope
// depth, so bracing is the one place the source's *shape* reaches this IL, and
// the recognizer pins every depth. The dc3 body is fully braced and so are
// these; the unbraced spelling of the same program is `n5` in the `_neg` file,
// where it is recorded as a REFUSAL of a program c2 almost certainly emits
// identically — the fence is narrower than the class, in the safe direction.

// p0 — the dc3 body, `==` spelling.
const Node * p0(Blend b, void *clip, float t) {
    const Node * n = 0;
    if (b >= b1) {
        if (b == b1) {
            n = 0;
        } else {
            if (b >= b2) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// p1 — the SAME program spelled `!(b != b1)`. Must be byte-identical to p0.
const Node * p1(Blend b, void *clip, float t) {
    const Node * n = 0;
    if (b >= b1) {
        if (!(b != b1)) {
            n = 0;
        } else {
            if (b >= b2) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// p2 — both literals moved. Exactly two words may differ from p0.
const Node * p2(int b, void *clip, float t) {
    const Node * n = 0;
    if (b >= 3) {
        if (b == 3) {
            n = 0;
        } else {
            if (b >= 7) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// p3 — a NEGATIVE first literal, so `cmpwi`'s immediate field is graded on both
// signs. `w6_rel_k.cpp` had twenty bodies and every one against a non-zero
// positive literal, which is how `Rel::Le`'s zero fold survived; the same trap
// one axis over.
const Node * p3(int b, void *clip, float t) {
    const Node * n = 0;
    if (b >= -1) {
        if (b == -1) {
            n = 0;
        } else {
            if (b >= 4) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// p4 — different callees and a different accumulator pointee type, so neither
// the two symbol names nor the pointee id can have been folded into the class.
int * p4(int b, void *clip, float t) {
    int * n = 0;
    if (b >= 1) {
        if (b == 1) {
            n = 0;
        } else {
            if (b >= 2) {
                n = ihi(clip, t);
            } else {
                n = ilo(clip, t);
            }
        }
    }
    return n;
}
