// W-MMIO3 — the ELISION's interprocedural half, as a whole-TU control.
//
// Byte for byte `wmmio3_close_call_chain.cpp` with ONE change: the callee of
// the statement the positive fixture ELIDES is an **external** rather than one
// of this TU's own functions. `w-ifn` #2351's D2 cell `e3` measured that c2
// KEEPS such a call, and this obj confirms it — `work/w-mmio3/ref/` has the
// `bl wmmio3s_setbuf` and its REL24 in `.text`.
//
// So the port must REFUSE, and the refusal is a WHOLE-TU one: the reader takes
// the body (it is the same grammar) and `c2_il::IlBundle::functions` then finds
// that the elided callee is not one of the names it bound. That is why this
// cell is its own file — a whole-TU `None` refuses every other cell in a file
// with it and grades none of them (`w-decouple` §8.2's shape), so a cell for
// this clause inside `wmmio3_close_call_chain_neg.cpp` would have been vacuous
// and would have looked exactly like nine working ones.
//
// **What grades it is the differential, not a census key.** `Port` must be
// `NotImplemented`; the must-fail mutation (`work/w-mmio3/mutate.sh` cell M1)
// deletes the WHOLE conjunction this clause is part of and turns it into
// `Port=Mismatch`, because the port then drops a `bl` and a relocation the obj
// has. A cell whose mutation only moves it from one refusal to another would
// not be a control at all.

typedef unsigned int uint;

struct wmmio3s_info;
typedef long (*wmmio3s_proc)(void *info, uint msg, long p1, long p2);

struct wmmio3s_info {
    uint dwFlags;
    uint fccIOProc;
    wmmio3s_proc proc;
};

extern "C" {
void wmmio3s_free(void *);
long wmmio3s_flush(void *h, uint f);
long wm3s_seek(void *h, long off, int origin);
long wmmio3s_close(void *h, uint u);
}

// NOT DEFINED HERE — this is the one change.
extern "C" long wmmio3s_setbuf(void *h, char *b, long c, uint f);

__declspec(noinline) long wmmio3s_flush(void *h, uint f) { return 0; }

long wm3s_seek(void *h, long off, int origin) { return 0; }

long wmmio3s_close(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3s_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3s_info *t = (wmmio3s_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;

    wmmio3s_setbuf(h, 0, 0, 0);
    wmmio3s_free(h);

    return 0;
}
