// **W-EXTDATA's fence, held.** Every function here is one clause of
// `shapes::guard_chain_shared_tail`'s accept boundary, broken in exactly one
// place, and every one must reach `NotImplemented` — never a `Mismatch`.
//
// A file-level `NotImplemented` is the CONJUNCTION and is satisfied by any one
// cell refusing, so it says nothing about the other cells. The per-cell verdicts
// are read with `c2rs census`, which reports one row per function, and each cell
// below is recorded with the clause it must decline on. Two cells declining on
// ONE clause is board **#1704**'s defect — a residue nobody can name is a residue
// nobody can size — so the clause names are distinct by construction and were
// checked per cell, not inferred from the file's verdict.
//
// **Every cell is braced exactly like the positive file**, which is w-cfgclass
// §6.2's confound: the recognizer pins every `54 <k>` scope depth, so an
// unbraced arm would refuse *on the bracing* and make a cell read as separating
// a clause it never reached.

typedef unsigned int usz;

typedef int (*outfn)(wchar_t *, usz, usz, const wchar_t *, void *, char *);
typedef int (*outfn5)(wchar_t *, usz, usz, const wchar_t *, void *);
typedef int (*ioutfn)(int *, usz, usz, const wchar_t *, void *, char *);

extern "C" {
extern int helper(outfn, wchar_t *, usz, usz, const wchar_t *, void *, char *);
extern int woutput(wchar_t *, usz, usz, const wchar_t *, void *, char *);
extern int helper5(outfn5, wchar_t *, usz, usz, const wchar_t *, void *);
extern int woutput5(wchar_t *, usz, usz, const wchar_t *, void *);
extern int ihelper(ioutfn, int *, usz, usz, const wchar_t *, void *, char *);
extern int iwoutput(int *, usz, usz, const wchar_t *, void *, char *);
extern int dhelper(int *, wchar_t *, usz, usz, const wchar_t *, void *, char *);
extern int gdata;
extern int *lasterr(void);
extern int *other(void);
extern void report(void);
extern void report2(void);
}

// n0 — FIVE formals, so the call takes six arguments and the rotate is one step
// shorter. `gcst-formals-not-6`. The arity is the clause that keeps this class a
// transcription: the `lis` sits after the second rotate step and, with one
// witness, "after the second" and "three before the last" are the same fact.
int n0(wchar_t *buffer, usz sizeInWords, usz count, const wchar_t *format, void *locale) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 0x16;
        report();
        return -1;
    }
    result = helper5(woutput5, buffer, sizeInWords, count, format, locale);
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

// n1 — a SIGNED guard formal. `gcst-guard-is-signed-so-c2-emits-cmpwi`.
// The emitter has one `cmplwi` per guard and no way to vary it, so a signed
// zero-compare would be the right program with a wrong word. Board #1706's rule
// (anything the emitter cannot vary must be refused by the READER) on the one
// axis that hides inside a TYPE rather than inside a token.
int n1(wchar_t *buffer, usz sizeInWords, int count,
       const wchar_t *format, void *locale, char *arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 0x16;
        report();
        return -1;
    }
    result = helper(woutput, buffer, sizeInWords, (usz)count, format, locale, arglist);
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

// n2 — the two error arms return DIFFERENT values.
// `gcst-arms-return-different-values`. The merged tail's `li r3,R` is emitted
// once and reached from both; two values are two tails and twelve more bytes.
int n2(wchar_t *buffer, usz sizeInWords, usz count,
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
    return -3;
}

// n3 — the two error arms call DIFFERENT reporters.
// `gcst-arms-call-different-reporter`. The `bl <invalid>` is in the shared tail,
// so two reporters is a second REL24 site the block plan has no room for.
int n3(wchar_t *buffer, usz sizeInWords, usz count,
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
    report2();
    return -1;
}

// n4 — a WORD store where the class has a halfword.
// `gcst-store-is-a-word-not-a-halfword`. `sth` and `stw` differ in one opcode
// field and in two bytes of what the program writes; nothing else in this file
// separates the store's width.
int n4(int *buffer, usz sizeInWords, usz count,
       const wchar_t *format, void *locale, char *arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 0x16;
        report();
        return -1;
    }
    result = ihelper(iwoutput, buffer, sizeInWords, count, format, locale, arglist);
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

// n5 — the call's first argument is a DATA address, not a function's.
// `gcst-fnaddr-no-decay`. The two are the SAME relocation quad and DIFFERENT
// symbol records (`Type` 0x0000 against 0x0020), which is the whole reason
// `IlFunction::fn_addr_sym` is a field of its own — measured side by side in one
// workload obj. Nothing in a mangled name tells them apart, so the reader tells
// them apart by the `2C` function-to-pointer decay, and this cell is what says
// the decay is required rather than tolerated.
int n5(wchar_t *buffer, usz sizeInWords, usz count,
       const wchar_t *format, void *locale, char *arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr() = 0x16;
        report();
        return -1;
    }
    result = dhelper(&gdata, buffer, sizeInWords, count, format, locale, arglist);
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

// n6 — the two error arms store through DIFFERENT nullary calls.
// `gcst-arms-call-different-errno`. Distinct from n3: that one moves the
// reporter in the shared tail, this one moves the call each arm makes BEFORE the
// merge, so the two arms' first words stop being the same instruction.
int n6(wchar_t *buffer, usz sizeInWords, usz count,
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
    *other() = 0x22;
    report();
    return -1;
}

// n7 — the two arms share their literal, so c2 merges them COMPLETELY and the
// body is shorter than this class emits. `gcst-arms-share-their-literal`.
// The one cell whose refusal is about the emitted body being too SMALL rather
// than too large, which is the direction a fence usually misses.
int n7(wchar_t *buffer, usz sizeInWords, usz count,
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
    *lasterr() = 0x16;
    report();
    return -1;
}
