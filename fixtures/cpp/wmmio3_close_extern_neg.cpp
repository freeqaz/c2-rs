// W-MMIO3 — the r31 PARK's interprocedural half, as a whole-TU control.
//
// Byte for byte `wmmio3_close_call_chain.cpp` with ONE change: the **void
// call's** callee is defined in this TU instead of being an external. That is
// the direction `w-ifn`'s `probe/park.cpp` measured from the other side — its
// `p2`/`p4` cells replace a same-TU callee with an external one and the park
// moves from r5 to r30 with the frame growing 96 → 112 — and it is what
// `WB_CHOOSER_FINDINGS` §2.3's M-RULE predicts: the register plan of this body
// is a function of the exact footprints of the calls a value is live across, so
// a callee whose footprint c2 knows exactly is not interchangeable with one
// whose footprint is the whole volatile set.
//
// The port must REFUSE, at `c2_il::IlBundle::functions`, whether or not c2's
// own plan happens to come out the same here — the transcription is graded on
// ONE witness and this is not it. That is the conservative direction and it is
// the only one this project tolerates.
//
// Its own file for `wmmio3_close_sibling_neg.cpp`'s reason: a whole-TU `None`
// refuses every cell in a file with it and grades none of them.

typedef unsigned int uint;

struct wmmio3x_info;
typedef long (*wmmio3x_proc)(void *info, uint msg, long p1, long p2);

struct wmmio3x_info {
    uint dwFlags;
    uint fccIOProc;
    wmmio3x_proc proc;
};

extern "C" {
void wmmio3x_free(void *);
long wmmio3x_flush(void *h, uint f);
long wmmio3x_setbuf(void *h, char *b, long c, uint f);
long wm3x_seek(void *h, long off, int origin);
long wmmio3x_close(void *h, uint u);
}

// DEFINED HERE — this is the one change.
__declspec(noinline) void wmmio3x_free(void *h) {}

__declspec(noinline) long wmmio3x_flush(void *h, uint f) { return 0; }
__declspec(noinline) long wmmio3x_setbuf(void *h, char *b, long c, uint f) { return 0; }

long wm3x_seek(void *h, long off, int origin) { return 0; }

long wmmio3x_close(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3x_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3x_info *t = (wmmio3x_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;

    wmmio3x_setbuf(h, 0, 0, 0);
    wmmio3x_free(h);

    return 0;
}
