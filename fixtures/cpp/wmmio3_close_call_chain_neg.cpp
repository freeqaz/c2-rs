// W-MMIO3 — the separating controls for `close_call_chain`. Every body here is
// ONE clause away from `wmmio3_close_call_chain.cpp` and every one must REFUSE,
// in the READER, at every mode.
//
// The point of the file is not that it refuses — a file of nonsense would do
// that. It is that each cell trips a **different** named clause, so the fence
// is exercised rather than merely present. `work/w-mmio3/NEG_CLAUSES.md`
// records the key each cell actually reached, read off `c2rs census` at the
// workload's own flags and **not predicted**.
//
// The two INTERPROCEDURAL clauses — the elided callee's purity and the parked
// callee's footprint — are NOT here and cannot be. They refuse at
// `IlBundle::functions`, which is a WHOLE-TU verdict, so a cell for one of them
// in this file would refuse every other cell with it and grade none of them
// (`docs/rungs/2026-08-09-w-decouple.md` §8.2's shape). They live one per file
// in `wmmio3_close_sibling_neg.cpp` and `wmmio3_close_extern_neg.cpp`, where
// the grading is the DIFFERENTIAL rather than a census key.

typedef unsigned int uint;

struct wmmio3n_info;
typedef long (*wmmio3n_proc)(void *info, uint msg, long p1, long p2);

struct wmmio3n_info {
    uint dwFlags;
    uint fccIOProc;
    wmmio3n_proc proc;
};

extern "C" {
void wmmio3n_free(void *);
long wmmio3n_flush(void *h, uint f);
long wmmio3n_setbuf(void *h, char *b, long c, uint f);
}

__declspec(noinline) long wmmio3n_flush(void *h, uint f) { return 0; }
__declspec(noinline) long wmmio3n_setbuf(void *h, char *b, long c, uint f) { return 0; }

// ---- n1: the guard's operand is an INT, not a pointer ----------------------
// c2 emits `cmpwi` here and `cmplwi` in the positive — one word apart, and it
// links either way, which is why the reader pins the operand's type.
extern "C" long wmmio3_n1(uint h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush((void *)h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf((void *)h, 0, 0, 0);
    wmmio3n_free((void *)h);
    return 0;
}

// ---- n2: the early return is BRACED ----------------------------------------
// One `53` and one `54 04` in the positive; two of each here. That is a
// different block plan and not a shallower spelling of the same one — it is
// exactly the difference between this class and `guard_ret_chain`'s arm.
extern "C" long wmmio3_n2(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0) {
        return r1;
    }
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n3: the early return tests `== 0`, not `!= 0` -------------------------
// `1F` where the class requires `20`. The branch sense inverts and the arm is
// no longer the value already in r3.
extern "C" long wmmio3_n3(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 == 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n4: the indirect call's third argument is a LITERAL --------------------
// The second formal is then dead and there is no r5 park at all. The whole
// register plan this class transcribes rests on that argument position.
extern "C" long wmmio3_n4(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, 7, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n5: the ELIDED call's result is USED ----------------------------------
// c2 keeps a call whose result is used (`w-ifn` D2 cell `e4`), so the statement
// is a `bl` the port would have to emit. The reader must not take the body.
extern "C" long wmmio3_n5(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    uint r3 = wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return r3;
}

// ---- n6: THREE formals -----------------------------------------------------
// A different arity moves every argument register.
extern "C" long wmmio3_n6(void *h, uint u, uint extra) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n7: the guard's arm returns a literal wider than a `li` immediate ------
extern "C" long wmmio3_n7(void *h, uint u) {
    if (h == 0) {
        return 70000;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n8: the indirect call goes through the FORMAL, not the cast local ------
// `mr r3,r31` in the positive is the cast local coming back out of the park;
// here the base and the first argument are two different values.
extern "C" long wmmio3_n8(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(h, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n9: the second early return returns a DIFFERENT value ------------------
// The arm is then not "the value already in r3" and costs an instruction, so
// the branch cannot fold into the jump to the epilogue.
extern "C" long wmmio3_n9(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r1;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free(h);
    return 0;
}

// ---- n10: the void call takes TWO arguments --------------------------------
extern "C" void wmmio3n_free2(void *, uint);
extern "C" long wmmio3_n10(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3n_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3n_info *t = (wmmio3n_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;
    wmmio3n_setbuf(h, 0, 0, 0);
    wmmio3n_free2(h, u);
    return 0;
}
