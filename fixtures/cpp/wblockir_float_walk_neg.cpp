// **w-blockir — the NEGATIVE cells of the float array-walk counted loop.**
//
// Every function here is one step outside `wblockir_float_walk.cpp` along
// exactly one axis, and **every one was compiled by real `c2.dll` under wibo
// before this file was written** — `work/w-blockir/probe/walk.cpp` and
// `probe/bound.cpp`, 24 cells, disassembly committed beside them. The right-hand
// column of each comment is what c2 *actually* emits, not what the reader
// happens to refuse.
//
// The discipline this file exists for: **an arm no cell grades is an arm that
// will be wrong when something finally reaches it** (board #1148). Eleven of the
// twelve cells below are shapes c2 lowers perfectly well, several of them
// one word from the accepted form, and the class refuses all of them because
// there is no *graded* emitter arm on the other side. `NotImplemented` is the
// only correct verdict for this whole file.
//
// Each cell names the reader clause that refuses it, and **the clauses are
// distinct** — verified by probe rather than asserted, because a `_neg` file
// whose cells all trip the same clause tests one thing eleven times. See
// `work/w-blockir/NEG_CLAUSES.md` for the per-cell key, taken from a reverted
// scratch print inside the production's own decline path. **Eleven of the
// twelve reach this reader and they trip TEN distinct clauses**; the twelfth
// (`n_noguard`) never reaches it, which is recorded rather than hidden.

namespace N {
    // ---- the OPERATION axis ------------------------------------------------

    // `-=` — c2 SWAPS THE TWO LOADS. The walker's own `lfs f0,0(r11)` comes
    // first and the other array's `lfsx f13,r10,r11` second, because the
    // non-commutative op pins its left operand (`probe/walk.cpp` cell `c7`,
    // 48 B). An arm that substituted only the A-form word would emit two loads
    // in the wrong order. Clause: `fwalk-compound-op`.
    void n_sub(unsigned int n, const float *a, float *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] -= a[i];
        }
    }

    // `/=` — the same swap with `fdivs` (cell `c8`, 48 B).
    // Clause: `fwalk-compound-op`, reached through a different opcode byte.
    void n_div(unsigned int n, const float *a, float *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] /= a[i];
        }
    }

    // ---- the TYPE axis -----------------------------------------------------

    // A **signed** counter and bound: c2 emits `cmpwi cr6,r3,0` and
    // `bclr 4,25` where the class has `cmplwi`/`bclr 12,26` — two different
    // words, and the two spellings differ in the IL by exactly ONE TYPE byte
    // (`86 41 74` against `86 42 75`) with the relational opcode and the branch
    // byte-identical. That is board #1788's fact; `readers::eat_int_like`
    // accepts both by design, so this class reads the signedness nibble instead.
    // Cell `c9`, 48 B. Clause: `fwalk-guard-type` — the refusal lands on the
    // GUARD's own type, before the counter is ever read.
    void n_signed(int n, const float *a, float *b) {
        if (n == 0)
            return;
        for (int i = 0; i < n; i++) {
            b[i] += a[i];
        }
    }

    // `double` arrays: `lfdx`/`lfd`/`fadd`/`stfd` and a stride of **8**, i.e.
    // four different words and a different immediate (cell `c11`, 48 B).
    // Clause: `fwalk-body-dst` — the subscript's scale literal is 8, not 4.
    void n_double(unsigned int n, const double *a, double *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
        }
    }

    // `int` arrays: `lwzx`/`lwz`/`add`/`stw` — **the skeleton generalises**, and
    // this lane does not ship the generalisation. Cell `c14`, 48 B, and the
    // guard, the `mtctr`, the `sub` and the `bdnz` are byte-identical to
    // `Add_InPlace`'s. Clause: `fwalk-body-rhs1-type` — the deref-load's TYPE
    // is an int, not a 4-byte real.
    void n_int(unsigned int n, const int *a, int *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
        }
    }

    // ---- the INDUCTION axis ------------------------------------------------

    // Step 2: c2 computes the trip count in the preheader —
    // `addi r10,r3,-1 · srwi r10,r10,1 · addi r10,r10,1` — and steps the walker
    // by 8 (cell `e3`, 60 B). `wb-loop` §9 item 4 leaves the selector between
    // `srwi` and `divwu` UNREAD. Clause: `fwalk-incr-lit-type` — `i += 2` is the
    // `0F` compound spelling where `i++` is `35`, so the stream diverges one
    // token BEFORE the step literal `fwalk-step-not-1` would have caught. The
    // clause that fires is the honest one and the comment says so rather than
    // the one the cell was designed for.
    void n_step2(unsigned int n, const float *a, float *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i += 2) {
            b[i] += a[i];
        }
    }

    // The counter used for something besides the subscript: a second live value,
    // `li r3,0`/`add r3,r9,r3`, a separate `addi r9,r9,1`, and an interleaved
    // schedule (cell `e1`, 68 B). Clause: `fwalk-then-return` — its guard is
    // `return 0;`, a VALUE return, so the then-clause diverges first. The cell
    // still grades what it says it grades: c2's body for it is 68 B and
    // nothing in this class could emit it.
    unsigned int n_ctru(unsigned int n, const float *a, float *b) {
        if (n == 0)
            return 0;
        unsigned int s = 0;
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
            s += i;
        }
        return s;
    }

    // ---- the SHAPE axis ----------------------------------------------------

    // The loop is not the function's tail: the body continues past the `bdnz`
    // with a REFHI/REFLO pair into `__real@3f800000` (cell `e2`, 60 B). The
    // whole twelve-word body depends on the loop being the tail — that is what
    // makes the guard a conditional RETURN rather than a forward branch.
    // Clause: `fwalk-for-scope-close`.
    void n_after(unsigned int n, const float *a, float *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
        }
        b[0] = 1.0f;
    }

    // The bound is a different formal from the guard's subject: c2 emits **two**
    // guards, `cmplwi cr6,r3,0 · bclr` then `cmplwi cr6,r4,0 · bclr` (cell `e4`,
    // 56 B). Clause: `fwalk-bound-not-formal0`.
    void n_bound(unsigned int n, unsigned int m, const float *a, float *b) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < m; i++) {
            b[i] += a[i];
        }
    }

    // Two statements in the loop body: a second `stfsx` inside the loop and
    // `bdnz .-24` (cell `e5`, 56 B). Clause: `fwalk-compound-arity` — four
    // formals where shape A takes three, which fires before the second
    // statement is ever reached. The comment names the clause that actually
    // fires rather than the one the cell was designed for.
    void n_two(unsigned int n, const float *a, float *b, float *c) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
            c[i] = b[i];
        }
    }

    // The two right-hand arrays in DESCENDING declaration order. c2 emits the
    // **byte-identical 52 bytes** it emits for `IPP::Mul` — same walker, same
    // two `sub`s, same everything (cell `c1`) — because the operation is
    // commutative and c2 normalises. The reader still refuses it, because what
    // selects the walker is declaration order and admitting the descending
    // spelling would mean deciding the walker from IL order instead. **This is
    // the cell where the accepted set is deliberately smaller than the set c2
    // converts.** Clause: `fwalk-binary-operands-descending`.
    void n_desc(unsigned int n, const float *a, const float *b, float *c) {
        if (n == 0)
            return;
        for (unsigned int i = 0; i < n; i++) {
            c[i] = b[i] * a[i];
        }
    }

    // The guard removed. c2 emits **exactly the same 48 bytes** as
    // `Add_InPlace` — the `for` rotation needs the zero-trip test anyway and c2
    // fuses the two (cell `c10`). So the `if (n == 0) return;` is redundant in
    // the obj and load-bearing in the IL: the token stream with it and the token
    // stream without it are different, and this class consumes the one it
    // graded. Clause: **none — this body never reaches this reader at all.**
    // Without the guard the body's first statement is `i = 0`, so the segment
    // dispatches on `26` into the OTHER arm of the ladder and this production is
    // not even asked. Recorded because it is the one cell whose refusal is a
    // dispatch fact rather than a clause, and a `_neg` file that did not say so
    // would be claiming a clause it does not exercise.
    void n_noguard(unsigned int n, const float *a, float *b) {
        for (unsigned int i = 0; i < n; i++) {
            b[i] += a[i];
        }
    }
}
