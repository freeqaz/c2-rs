// **W-CFG1's fence, held.** Every function here is one clause of
// `shapes::if_call_join`'s accept boundary, broken in exactly one place, and
// every one must reach `NotImplemented` — never a `Mismatch`.
//
// A file-level `NotImplemented` is the CONJUNCTION and is satisfied by any one
// cell refusing, so it says nothing about the other cells. The per-cell verdicts
// are read with `c2rs census`, which reports one row per function.
//
// **Every cell here is braced exactly like the positive file**, and that is the
// confound this rung was warned about (w-clear, bitten twice): the recognizer
// pins every `54 <k>` scope depth, so an unbraced arm refuses *on the bracing*
// and would make a cell read as separating a clause it never reached. `n5` is
// the cell that isolates the bracing itself, so the other five cannot be
// confounded by it.
//
// The rival this separates is named in `work/w-cfgclass/GRID.md`: a general
// one-compare/two-guard/two-arm lowering agrees with the shipped class on every
// POSITIVE cell and disagrees on every one of these.

struct Node;
extern const Node *hi(void *, float);
extern const Node *lo(void *, float);
extern const Node *nofp(void *);

// n0 — THREE formals, so the arity clause cannot fire, and the two arms call
// with DIFFERENT first arguments. The hoist is then illegal: c2 puts a setup
// back inside each arm and the entry block loses its `mr r3,r4`.
//
// This cell was first written with a fourth formal, and the recognizer declined
// it `ifjoin-formals-not-3` — the arity clause, not the clause under test. That
// is w-clear's confound reproduced inside the file whose header warns about it,
// and it was found by printing the recognizer's own decline context per cell
// rather than by reading the census, which reports only the fall-through
// blocker and would have shown this cell as refusing exactly like the others.
const Node *n0(int b, void *clip, float t) {
    const Node *n = 0;
    if (b >= 1) {
        if (b == 1) {
            n = 0;
        } else {
            if (b >= 2) {
                n = hi(clip, t);
            } else {
                n = lo(0, t);
            }
        }
    }
    return n;
}

// n1 — the dead arm stores a DIFFERENT value, so the middle block is not empty,
// c2 emits it, and both branch senses invert back.
const Node *n1(int b, void *clip, float t) {
    const Node *n = 0;
    if (b >= 1) {
        if (b == 1) {
            n = hi(clip, t);
        } else {
            if (b >= 2) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// n2 — TWO formals, no `float`. The park and the hoist are a register
// assignment for the three-formal arity and nothing here has been graded at
// another.
const Node *n2(int b, void *clip) {
    const Node *n = 0;
    if (b >= 1) {
        if (b == 1) {
            n = 0;
        } else {
            if (b >= 2) {
                n = nofp(clip);
            } else {
                n = nofp(clip);
            }
        }
    }
    return n;
}

// n3 — the accumulator is a FILE-SCOPE pointer. `li r11,0` would fold away a
// real memory store.
static const Node *g_n;
const Node *n3(int b, void *clip, float t) {
    g_n = 0;
    if (b >= 1) {
        if (b == 1) {
            g_n = 0;
        } else {
            if (b >= 2) {
                g_n = hi(clip, t);
            } else {
                g_n = lo(clip, t);
            }
        }
    }
    return g_n;
}

// n4 — the middle test is `<` rather than `==`/`!=`, **against the SAME literal
// the outer test uses**, so the only thing that differs is the relation. Written
// first as `b < 5`, it declined `ifjoin-mid-lit-differs` — the shared-literal
// clause, not the relation clause. Same confound as n0, same instrument found
// it.
const Node *n4(int b, void *clip, float t) {
    const Node *n = 0;
    if (b >= 1) {
        if (b < 1) {
            n = 0;
        } else {
            if (b >= 2) {
                n = hi(clip, t);
            } else {
                n = lo(clip, t);
            }
        }
    }
    return n;
}

// n5 — the SAME PROGRAM as the positive file's `p0`, with the two innermost arms
// unbraced. It refuses, and this cell exists so that refusal is a recorded
// property of the fence rather than a surprise: `54 <k>` carries the scope depth
// and the recognizer pins every one, so bracing is visible to it where it is
// almost certainly invisible to `.text`. **The fence is NARROWER than the class
// c2 has**, in the safe direction, and widening it needs its own graded cells.
const Node *n5(int b, void *clip, float t) {
    const Node *n = 0;
    if (b >= 1) {
        if (b == 1) {
            n = 0;
        } else {
            if (b >= 2) n = hi(clip, t);
            else        n = lo(clip, t);
        }
    }
    return n;
}
