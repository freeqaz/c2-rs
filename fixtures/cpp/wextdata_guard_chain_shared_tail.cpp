// **W-EXTDATA — a `||` guard chain SUNK to the end of the function and
// TAIL-MERGED with a second error block, around one call whose first argument is
// the ADDRESS OF A FUNCTION.** The port's first REFHI/REFLO against a `Type
// 0x0020` symbol, and its first `lis` that is not the body's first word.
//
// `src/xdk/LIBCMT/vswprnc.cpp` is `p0` almost verbatim and it is a FRONTIER TU
// with exactly one emitted function, so it converts on this class or on none.
// This file is the class's FENCE — the workload contains one instance and one
// instance cannot tell a rule from a coincidence (board #260).
//
//     mflr/stw/std r31/stwu -96
//     mr    r31,r3        the park: `a0` is stored through after two `bl`s
//     mr    r9,r8         the hoist: the LAST rotate step, above every branch
//     cmplwi cr6,rX,0     ┐ THREE branches to ONE block. The `||` is not a
//     bt    26,Lerr       │ computed boolean, and the tested formals are named
//     cmplwi cr6,rY,0     │ in test order, so their registers are too
//     bt    26,Lerr       │
//     cmplwi cr6,rZ,0     │
//     bt    26,Lerr       ┘
//     mr r8,r7 ; mr r7,r6 ; lis r11 ; mr r6,r5 ; mr r5,r4 ; mr r4,r3
//                         the 5-deep rotate with the REFHI INTERLEAVED at
//                         word 14 — WR1's "the `lis` is the first word" is
//                         false here by thirteen words
//     addi  r3,r11,0      the REFLO
//     bl    <helper>
//     cmpwi cr0,r3,0      `r < 0` on cr0 …
//     bf    0,Lskip
//     li r11,0 ; sth r11,0(r31)          a HALFWORD store
//     cmpwi cr6,r3,S      … and `r != S` on cr6 — a DIFFERENT condition reg
//     bf    26,epilogue
//     bl <errno> ; li r11,K_RANGE ; b Ltail      the RANGE arm
//  Lerr:
//     bl <errno> ; li r11,K_GUARD                the GUARD arm, sunk here
//  Ltail:
//     stw r11,0(r3) ; bl <invalid> ; li r3,R     the MERGED TAIL, emitted ONCE
//     addi/lwz/mtlr/ld r31/blr
//
// The STRUCTURAL axes are held by `wextdata_guard_chain_shared_tail_neg.cpp`;
// what varies here is the VALUE axis. Board #198's rule in both directions.
//
// **Every arm is braced, and that is load-bearing.** `54 <k>` carries the scope
// depth, so bracing is the one place the source's *shape* reaches this IL, and
// the recognizer pins every depth. The dc3 body is fully braced and so are these.

typedef unsigned int usz;

typedef int (*outfn)(wchar_t *, usz, usz, const wchar_t *, void *, char *);
typedef int (*outfn2)(unsigned short *, usz, usz, const unsigned short *, void *, char *);

extern "C" {
extern int helper(outfn, wchar_t *, usz, usz, const wchar_t *, void *, char *);
extern int woutput(wchar_t *, usz, usz, const wchar_t *, void *, char *);
extern int helper2(outfn2, unsigned short *, usz, usz, const unsigned short *, void *, char *);
extern int woutput2(unsigned short *, usz, usz, const unsigned short *, void *, char *);
extern int *lasterr(void);
extern int *lasterr2(void);
extern void report(void);
extern void report2(void);
}

// p0 — the dc3 body. `count` is tested first, then `buffer`, then `sizeInWords`,
// which is what puts `cmplwi` on r5, r3, r4 in that order.
int p0(wchar_t *buffer, usz sizeInWords, usz count,
       const wchar_t *format, void *locale, char *arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 0x16;
        report();
        return -1;
    }
    result = helper(woutput, buffer, sizeInWords, count, format, locale, arglist);
    if (result < 0) {
        *buffer = 0;
    }
    if (result != -2) {
        return result;
    }
    *lasterr() = 0x22;
    report();
    return -1;
}

// p1 — every one of the four immediates moved, and the sentinel made POSITIVE.
// `cmpwi`'s field is graded on both signs; `w6_rel_k.cpp` had twenty bodies and
// every one against a non-zero positive literal, which is how `Rel::Le`'s zero
// fold survived. Exactly four words may differ from `p0`.
int p1(wchar_t *buffer, usz sizeInWords, usz count,
       const wchar_t *format, void *locale, char *arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 5;
        report();
        return -7;
    }
    result = helper(woutput, buffer, sizeInWords, count, format, locale, arglist);
    if (result < 0) {
        *buffer = 0;
    }
    if (result != 9) {
        return result;
    }
    *lasterr() = 11;
    report();
    return -7;
}

// p2 — the guard ORDER permuted (`buffer`, `sizeInWords`, `count`) and all four
// callees different. Three `cmplwi` operands move and four symbols change; the
// block plan must not. Nothing else in this corpus grades the guard order, and
// it is the one value axis that reaches an *instruction operand* rather than an
// immediate — a class that had folded the workload's order into the emitter
// would pass every other cell here.
int p2(unsigned short *buffer, usz sizeInWords, usz count,
       const unsigned short *format, void *locale, char *arglist) {
    int result;
    if (buffer == 0 || sizeInWords == 0 || count == 0) {
        *lasterr2() = 0x16;
        report2();
        return -1;
    }
    result = helper2(woutput2, buffer, sizeInWords, count, format, locale, arglist);
    if (result < 0) {
        *buffer = 0;
    }
    if (result != -2) {
        return result;
    }
    *lasterr2() = 0x22;
    report2();
    return -1;
}
