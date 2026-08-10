// W-MMIO3 — the guarded close chain: the class that `src/xdk/nuispeech/mmio.cpp`'s
// `mmioClose` is the workload instance of, and the last 124 of that TU's 380
// bytes. `w-ifn` shipped the other two bodies (`wifn_guard_ret_chain.cpp`) and
// declined this one at six mechanisms.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately out of class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself — five things no earlier
// fixture in this seam can grade:
//
//  * **AN INDIRECT CALL.** `mtctr`/`bctrl` through a function pointer loaded
//    out of a member. It is the first call this port emits whose callee the IL
//    never names — not by a `.gl` token, not by an intrinsic selector, not at
//    all — so it is also the first REL24-free call site and the first body
//    whose branch target is a register.
//  * **AN ELIDED CALL.** `wmmio3_setbuf(h, 0, 0, 0)` IS in the `.ex` stream and
//    the obj carries no branch, no relocation and no symbol for it. c2 deletes
//    a call whose result is unused when the callee is defined in this TU with a
//    side-effect-free body (`w-ifn` #2351's D2, ten cells at `/O1` and again at
//    `/Ob0`). This is the first fixture where the port must emit FEWER calls
//    than the source spells, and the first whose acceptance depends on a fact
//    about a DIFFERENT function's body.
//  * **A PARK IN A VOLATILE ACROSS A `bl`.** `u` lives in **r5** — an argument
//    register, a caller-save — from the prologue to the `bctrl`, across the
//    call to `wmmio3_flush`. That is only correct because c2 reads the callee's
//    exact register footprint (`WB_CHOOSER_FINDINGS` §2.3, M-RULE) and r5 is
//    what the `bctrl`'s third argument wants ("coalescing beats allocation").
//    Every other park this port emits is into a callee-saved register.
//  * **TWO CONDITION-REGISTER FIELDS IN ONE BODY, ON THE SAME OPCODE.** The
//    guard is `cmplwi cr6,r3,0` = `2b030000` and the two result tests are
//    `cmplwi cr0,r3,0` = `28030000`. Four bits apart, and the difference is
//    "compare a formal" against "compare a call's return value".
//  * **A BRANCH WHOSE SENSE IS INVERTED AGAINST THE SOURCE.** `if (r1 != 0)
//    return r1;` becomes a branch taken when `r1` is NON-zero, straight to the
//    epilogue, because the value it returns is already in r3 and the arm
//    therefore costs no instruction at all.
//
// **`wm3_seek` is not decoration.** Its name is EIGHT bytes and carries no
// `@@`, which is exactly what stops `gl::gl_defined_names`' narrow walk on
// `mmio.cpp` (`mmioSeek`, same length, same reason). With it here the FENCE
// EXEMPTION's incumbent ground set is empty and the TU is licensed only by
// `gl::plain_external_names_among`, which is this lane's mechanism 7. Delete
// this one function and the fixture stops grading that mechanism while still
// passing — which is the shape `docs/GAPS.md` warns about, so it is said here.

typedef unsigned int uint;

struct wmmio3_info;
typedef long (*wmmio3_proc)(void *info, uint msg, long p1, long p2);

struct wmmio3_info {
    uint dwFlags;     // 0x0
    uint fccIOProc;   // 0x4
    wmmio3_proc proc; // 0x8
};

// The declarations go in an `extern "C"` block and the `__declspec` on the
// DEFINITION, which is how `src/xdk/nui/mmio.h` and `mmio.cpp` do it — and it
// is not a style choice. Written the other way round,
// `__declspec(noinline) extern "C" long f(…)`, the attribute is silently
// dropped: measured here, first try, `work/w-mmio3/ref/fx1_bad.obj` has NO
// `bl wmmio3_flush` at all, c2 having inlined the eight-byte callee and then
// folded the `if (r1 != 0)` that tested its constant result. The obj is 104
// bytes where this one is 124, and nothing warns.
extern "C" {
void wmmio3_free(void *);
long wmmio3_flush(void *h, uint f);
long wmmio3_setbuf(void *h, char *b, long c, uint f);
long wm3_seek(void *h, long off, int origin);
long wmmio3_close(void *h, uint u);
}

// The call the r5 park crosses. `__declspec(noinline)` for the reason the
// workload's own `mmioFlush` carries it: an eight-byte callee is inside every
// size-keyed inline bracket there is, and the attribute is the only thing that
// makes c2 keep the `bl` — `w-mmioclose` reads it out of the `.gl` record's
// attribute byte (bit `0x40`) and `c2_core::comdat::fenced_inlined_callee`
// asks it AHEAD of the size test, because legality precedes profitability.
__declspec(noinline) long wmmio3_flush(void *h, uint f) { return 0; }

// The ELIDED call's callee. Also `noinline`, mirroring the workload — and note
// that the attribute is NOT what makes this one disappear: D2's cell `e5`
// elides without it and `e6` elides a callee that emits real bytes. What makes
// it disappear is that its result is unused and its body has no side effect.
__declspec(noinline) long wmmio3_setbuf(void *h, char *b, long c, uint f) {
    return 0;
}

// The eight-byte undecorated defined name. See the header comment.
long wm3_seek(void *h, long off, int origin) { return 0; }

long wmmio3_close(void *h, uint u) {
    if (h == 0) {
        return 5;
    }
    uint r1 = wmmio3_flush(h, 0);
    if (r1 != 0)
        return r1;
    wmmio3_info *t = (wmmio3_info *)h;
    uint r2 = t->proc(t, 4, u, 0);
    if (r2 != 0)
        return r2;

    wmmio3_setbuf(h, 0, 0, 0);
    wmmio3_free(h);

    return 0;
}
