// **MUST REFUSE — lane `w-pool2` (#2592).** Seven cells, each one axis out of
// `wpool2_free_list.cpp`, each on a clause this lane pinned deliberately.
//
// Every cell here was read with the probe rather than by inspection —
// `w-biquad` #2535 had SEVEN of eleven cells confounded by source formatting,
// caught by running them and not by reading them, and this file's per-cell keys
// are asserted in `crates/c2-harness/tests/pool2_cells.rs`.
//
// Two of the seven are backed by an obj that shows real `c2` emitting something
// the port could not, which is the difference between a refusal and caution:
//
//   * **N1** — `return (void *)1` instead of `return nullptr`. `c2` emits
//     **36 bytes**: `bf 26,+12 ; li r3,1 ; blr` before the fall-out block. The
//     guard stops folding to `bclr` ENTIRELY. Without the literal-zero clause
//     the port would emit 28 bytes for a 36-byte function.
//     (`work/w-pool2/probe/p_ret1.obj`)
//   * **N5** — `count > 0` instead of `count > 1`. `c2` emits **76 bytes** and
//     a record-form **`divw.`**, folding the comparison back into the
//     division's own opcode and branching on cr0 (`bf 1`) instead of cr6. The
//     guard literal reaches into the *division's* instruction selection.
//     (`work/w-pool2/probe/p_gt0.obj`)
//
// The remaining five pin structure rather than a rival obj: an arity the
// register plan was never measured at, a store order, a second member, an
// element scale, and a round-up alignment whose addend and `rlwinm` mask are a
// matched pair with only one witness.

struct WP2 {
    char *mFree;
    char *mOther;
};

// ---- N1 — the POP's guarded arm returns a NON-ZERO literal ----------------
// The whole reason POP's guarded arm is free is that the popped head is already
// in r3 and the guard proves it is 0 there. A different literal has to be
// materialised and the fold is lost.
void *wpool2_n1_ret_one(WP2 *p) {
    void *ptr = p->mFree;
    if (!ptr)
        return (void *)1;
    p->mFree = *(char **)ptr;
    return ptr;
}

// ---- N2 — the PUSH's two stores in the OTHER order ------------------------
// `mFree = v` before `*v = mFree` is a different program and a different obj:
// the link written into `*v` is the one just stored. The run is pinned
// statement by statement against the base token, so the order is part of the
// class rather than a property the emitter would rediscover.
void wpool2_n2_push_reordered(WP2 *p, void *v) {
    if (!v) {
        return;
    }
    p->mFree = (char *)v;
    *(void **)v = p->mFree;
}

// ---- N3 — the PUSH touches TWO different members --------------------------
// One `lwz` displacement and one `stw` displacement, and the recognizer
// requires the body's two designators to agree on it. Two members is a second
// displacement and a register plan nothing here graded.
void wpool2_n3_two_members(WP2 *p, void *v) {
    if (!v) {
        return;
    }
    *(void **)v = p->mOther;
    p->mFree = (char *)v;
}

// ---- N4 — the PUSH carries a SECOND formal --------------------------------
// The formal's slot index IS its register, and the plan was measured at one
// formal. A second one occupies r5 and changes nothing visible about this body,
// which is exactly why it is refused rather than assumed harmless.
//
// **The third parameter is UNNAMED, and its first draft was not.** With
// `int unused` and a `(void)unused;` discard, the cell carried an extra IL
// statement and the arity counterfactual read IMPRECISE — the cell stayed
// blocked with the arity clause relaxed, because a second clause was holding
// it. That is `w-biquad` #2535's confound in this lane's own `_neg` file,
// caught by running `work/w-pool2/neg_clauses.py` and not by reading the cell.
void wpool2_n4_second_formal(WP2 *p, void *v, int) {
    if (!v) {
        return;
    }
    *(void **)v = p->mFree;
    p->mFree = (char *)v;
}

struct WP2C {
    char *mFree;
};

// ---- N5 — the constructor's guard literal is 0, not 1 ---------------------
// See the header: this is the cell that makes `c2` emit `divw.`.
struct WP2G {
    char *mFree;
    WP2G(int i1, void *v, int i2);
};
WP2G::WP2G(int i1, void *v, int i2) : mFree((char *)v) {
    char *ptr = (char *)v;
    int stride = (i1 + 3) & ~3;
    int count = i2 / stride;
    if (count > 0) {
        int n = count - 1;
        do {
            char *next = ptr + stride;
            *(char **)ptr = next;
            ptr = next;
        } while (--n);
    }
    *(char **)ptr = 0;
}

// ---- N6 — the constructor rounds up to EIGHT, not four --------------------
// The `+ (align-1)` addend and the `rlwinm` MB/ME pair are a MATCHED pair and
// this lane graded exactly one of them. Refused in the recognizer, so the
// census and the gate cannot disagree about it.
struct WP2A {
    char *mFree;
    WP2A(int i1, void *v, int i2);
};
WP2A::WP2A(int i1, void *v, int i2) : mFree((char *)v) {
    char *ptr = (char *)v;
    int stride = (i1 + 7) & ~7;
    int count = i2 / stride;
    if (count > 1) {
        int n = count - 1;
        do {
            char *next = ptr + stride;
            *(char **)ptr = next;
            ptr = next;
        } while (--n);
    }
    *(char **)ptr = 0;
}

// ---- N7 — the walking pointer's element scale is not 1 --------------------
// `long *` scales the stride by 4, which is a `mulli`/`slwi` inside the loop
// body that this class has no witness of. The IL spells the scale as the `33 1`
// before the `04`, and the recognizer requires it to be exactly 1.
struct WP2S {
    long *mFree;
    WP2S(int i1, void *v, int i2);
};
WP2S::WP2S(int i1, void *v, int i2) : mFree((long *)v) {
    long *ptr = (long *)v;
    int stride = (i1 + 3) & ~3;
    int count = i2 / stride;
    if (count > 1) {
        int n = count - 1;
        do {
            long *next = ptr + stride;
            *(long **)ptr = next;
            ptr = next;
        } while (--n);
    }
    *(long **)ptr = 0;
}
