// W9 — the SIGNED half of the conditional-branch relation grid.
//
// W8 (`w8_cond_tail.cpp`) shipped the port's first conditional branch and it is
// byte-exact. But every W8 fixture tests exactly one cell: `v1 == 0` on a
// **pointer**. That is `Rel::Eq` on an **unsigned** operand against the literal
// **0** — so `branch_sense`'s other five rows and the whole `cmpwi` path have
// never met the real `c2`. They are asserted only by a unit test comparing the
// port's table to itself, which is `docs/STATUS.md` trap 5 ("absence reads as
// success unless something forbids it") and board #137's shape.
//
// Lane w-frame's ranking measured that this is the frontier's most-wanted
// construct, not an afterthought: `bt` — the true-sense branch this file is the
// first to demand — is missing from **8 of the 17 FRONTIER TUs** and `cmpwi`
// from **6**. Both are the top two entries in `work/w-frame/RANKING.md` §3.2.
//
// The shape is w8_cond_tail.cpp's, changed in as few places as possible so a
// failing cell localizes:
//
//   * the scrutinee is the THIRD formal (r5), not the first, so neither arm
//     wants its register and no entry-block park is involved. The compare reads
//     a home register, exactly as `?MemFree`'s does;
//   * both arms still tail-call a DIFFERENT external, which is what keeps every
//     body inside fold band 3 by construction (docs/CFG_SHAPE.md §3.5) rather
//     than by reproducing the cost model that section declines (board #187);
//   * the operand is `int`, so the compare must be `cmpwi` (2f……) and not
//     `cmplwi` (2b……) — the relational opcodes are sign-agnostic and only the
//     operand type triple says which (§3.2).
//
// Six relations, one per function. Expected per `docs/CFG_SHAPE.md` §3.1 and
// `branch_sense`: the emitted branch is the NEGATION of the source relation,
// because the IL's `38` is brFALSE — `==` gives `bne`, `!=` gives `beq`, `<`
// gives `bge`, `>=` gives `blt`, `>` gives `ble`, `<=` gives `bgt`. The three
// with `BO=12` (`bt`) have no oracle witness anywhere in the corpus today.
void g2(void *, unsigned long);
void h3(void *, unsigned long, void *);

void s_eq(void *v1, unsigned long ul, int a) {
    if (a == 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_ne(void *v1, unsigned long ul, int a) {
    if (a != 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_lt(void *v1, unsigned long ul, int a) {
    if (a < 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_ge(void *v1, unsigned long ul, int a) {
    if (a >= 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_gt(void *v1, unsigned long ul, int a) {
    if (a > 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_le(void *v1, unsigned long ul, int a) {
    if (a <= 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}
