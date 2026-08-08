// CONSTRUCT LADDER — from the shipped class to `mmio.cpp`'s `mmioClose`.
typedef unsigned int uint;
struct MI2;
typedef unsigned long (*IOPROC)(MI2 *, unsigned int, unsigned int, unsigned int);
struct MI2 { char pad[8]; IOPROC pIOProc; };
void FreeHandle2(void *);
void ext1(void *, unsigned int);
__declspec(noinline) unsigned long p_flush(void *h, unsigned int f) { return 0; }
__declspec(noinline) unsigned long p_setbuf(void *h, char *b, long c, unsigned int f) { return 0; }

// C0 — one guard, one call with a literal slot. In class at this lane's tip.
unsigned long C0(void *a0, unsigned int a1) {
    if (a0 == 0) return 5;
    ext1(a0, 0);
    return 0;
}

// C1 — C0 with the call's RESULT tested and returned. A guard AFTER a call, on
// a call result, which c2 compares on **cr0** where every formal guard is cr6.
unsigned long C1(void *a0, unsigned int a1) {
    if (a0 == 0) return 5;
    uint r = p_flush(a0, 0);
    if (r != 0) return r;
    return 0;
}

// C2 — C1 with a second, INDIRECT call through a loaded member (`lwz` +
// `mtctr` + `bctrl`), whose result is tested the same way.
unsigned long C2(void *a0, unsigned int a1) {
    if (a0 == 0) return 5;
    uint r = p_flush(a0, 0);
    if (r != 0) return r;
    MI2 *info = (MI2 *)a0;
    uint q = info->pIOProc(info, 4, a1, 0);
    if (q != 0) return q;
    return 0;
}

// C3 — C2 with the trailing external call. `mmioClose` minus the ELIDED one.
unsigned long C3(void *a0, unsigned int a1) {
    if (a0 == 0) return 5;
    uint r = p_flush(a0, 0);
    if (r != 0) return r;
    MI2 *info = (MI2 *)a0;
    uint q = info->pIOProc(info, 4, a1, 0);
    if (q != 0) return q;
    FreeHandle2(a0);
    return 0;
}

// C4 — C3 with the call c2 ELIDES: `p_setbuf`'s result is unused and its body
// is a constant, and the obj carries no branch for it at all.
unsigned long C4(void *a0, unsigned int a1) {
    if (a0 == 0) return 5;
    uint r = p_flush(a0, 0);
    if (r != 0) return r;
    MI2 *info = (MI2 *)a0;
    uint q = info->pIOProc(info, 4, a1, 0);
    if (q != 0) return q;
    p_setbuf(a0, 0, 0, 0);
    FreeHandle2(a0);
    return 0;
}
