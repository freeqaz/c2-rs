// W-OSFINFO — the NEGATIVE controls for `osf-handle-guard`.
//
// Ten cells, each one clause away from `wosf_handle_guard.cpp`, and **every one
// must be out of class**. A `_neg` file whose cells all decline for the SAME
// reason is a control that tests one clause ten times, which is why each cell's
// clause was read individually with a probe patch (applied, run and reverted —
// `work/w-osfinfo/decline_probe.md`, board #1704's defect and w-cfgclass §6.2's
// method).
//
// Compile at `/O1 /Oi /EHsc /GR`, the class's own mode. At `/Ox` every cell is
// out of class for an eleventh reason — the mode gate in the parser — which is
// the positive fixture's second row and not a cell here.

struct ioinfo {
    long hnd;
    char osfile;
    char pad[67];      // 72 bytes, not a power of two
};

// n5's element: 64 bytes, a power of two, so c2 scales with `slwi`.
struct ioinfo_pow2 {
    long hnd;
    char osfile;
    char pad[59];
};

// n4's element: the handle is NOT at offset 0.
struct ioinfo_off {
    long lead;
    long hnd;
    char osfile;
    char pad[63];
};

// n3's element: the flag is a WORD.
struct ioinfo_word {
    long hnd;
    long osfile;
    char pad[64];
};

extern int nhandle;
extern ioinfo *pioinfo[];
extern ioinfo_pow2 *pioinfo_pow2[];
extern ioinfo_off *pioinfo_off[];
extern ioinfo_word *pioinfo_word[];

extern "C" int *c2rs_errno_a();
extern "C" int *c2rs_errno_b();
extern "C" int *c2rs_errno_arg(int which);

// n1 — the formal is a SIGNED int, so the first guard carries no `2C` and c2
// emits one compare form for both guards where this class has two.
int n1(int fh)
{
    if (fh >= 0 && (unsigned) fh < (unsigned) nhandle) {
        int i = fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n2 — the RANGE guard is SIGNED: the conversion goes the other way, so c2
// emits `cmpw` where this class has `cmplw`. The right program, one wrong word.
int n2(unsigned fh)
{
    if ((int) fh >= 0 && (int) fh < nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n3 — the flag member is a WORD, so c2 emits `lwz` where this class has `lbz`.
int n3(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo_word *e = &pioinfo_word[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n4 — the handle member is NOT at offset 0, so the success store and the error
// store are two words and the tail merge does not exist.
int n4(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo_off *e = &pioinfo_off[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n5 — the element size is a power of two, so c2 scales with `slwi` where this
// class has `mulli`.
int n5(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo_pow2 *e = &pioinfo_pow2[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n6 — the flag mask is not `2^n − 1`, so there is no `clrlwi` for it.
int n6(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 6) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n7 — the stored sentinel is not the compared one. The emitter drives the
// `cmpwi` and the `li` from ONE field and has no way to vary them apart.
int n7(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -2;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n8 — the two inner guards NESTED instead of short-circuited. Two extra scope
// opens, and a different block plan.
int n8(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0) {
            if (e->hnd != -1) {
                e->hnd = -1;
                return 0;
            }
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n9 — a non-static MEMBER function: `this` occupies r3 and the formal moves to
// r4, so every register in the thirty-one words is wrong.
struct N9 {
    int f(unsigned fh);
};

int N9::f(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}

// n10 — an error call that takes an ARGUMENT. The class's frame is 96 bytes
// because both calls are nullary, and the argument setup is a `li` it has no
// word for.
int n10(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        ioinfo *e = &pioinfo[i][fh & 31];
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_arg(1) = 9;
    *c2rs_errno_b() = 0;
    return -1;
}
