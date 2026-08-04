// W9 — the UNSIGNED half of the conditional-branch relation grid, and the
// first NON-ZERO comparison literal in the corpus.
//
// Companion to `w9_rel_signed.cpp`; same body shape, two axes moved:
//
//   * the operand is `unsigned int`, so the compare stays `cmplwi` (2b……) —
//     this half holds the sign axis fixed at the value W8 already witnessed and
//     varies only the relation;
//   * the literal is **7**, not 0. Every comparison the port has ever emitted
//     against the real `c2` used the literal 0, so the immediate field of
//     `cmplwi`/`cmpwi` is graded here for the first time. It is also the cell
//     that would catch a canonicalization — if `c2` rewrites `u > 7` as
//     `u >= 8` the emitted immediate moves and the port is wrong about a byte
//     it currently believes it controls.
//
// `u < 0` and `u >= 0` would be degenerate at an unsigned zero (always false /
// always true) and would grade a fold rather than a branch, which is why this
// half uses 7 rather than mirroring the signed file's literal.
//
// Both arms still tail-call a different external: fold band 3 by construction.
void g2(void *, unsigned long);
void h3(void *, unsigned long, void *);

void u_eq(void *v1, unsigned long ul, unsigned int a) {
    if (a == 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void u_ne(void *v1, unsigned long ul, unsigned int a) {
    if (a != 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void u_lt(void *v1, unsigned long ul, unsigned int a) {
    if (a < 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void u_ge(void *v1, unsigned long ul, unsigned int a) {
    if (a >= 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void u_gt(void *v1, unsigned long ul, unsigned int a) {
    if (a > 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void u_le(void *v1, unsigned long ul, unsigned int a) {
    if (a <= 7) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}
