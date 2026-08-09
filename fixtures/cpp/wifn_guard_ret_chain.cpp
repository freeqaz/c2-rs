// W-IFN — the framed guard chain whose arms are `return K` and whose spine is
// a block copy. Both sub-shapes of the class that `src/xdk/nuispeech/mmio.cpp`'s
// `mmioGetInfo` and `mmioSetInfo` are the workload instances of.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/2 in class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself — three things no earlier
// fixture in this seam can grade:
//
//  * **A CALL WHOSE CALLEE THE IL NEVER NAMES.** `memcpy` arrives as intrinsic
//    selector 172 on a `40` token; the captured `.gl` for `mmio.cpp` contains
//    no `memcpy` string at all while the obj carries it as an undefined
//    external. Every other REL24 this port emits is a `.gl` token resolved
//    through `IlFunction::callees`. This is the first minted one, so it is also
//    the first cell that grades the symbol-table PLACEMENT of a name that has
//    no `.gl` record.
//  * **TWO forward guard arms branching into ONE materialised epilogue.**
//    `Selected::Framed` emits prologue + body + epilogue as one straight run
//    with no representation for a join at all (board #506). Both bodies here
//    reach the epilogue from three places.
//  * **The SECOND relational regime in the same body as the first.** The two
//    guards read crf6's EQ bit (`bf 26`) and the clamp in `wifn_set_info` reads
//    crf6's LT bit (`bf 24`), off a `cmplw` on two LOADED values rather than a
//    `cmplwi` against a literal. One CR field, two bits, four words apart.
//
// The two sub-shapes differ by which formal is the copy's DESTINATION, and that
// one bit re-plans the whole body: `wifn_get_info` swaps its two pointer
// formals through r11 and saves no GPR, `wifn_set_info` parks its destination
// in r31 and saves one. They are transcribed separately for that reason.

typedef unsigned int uint;

extern "C" void *memcpy(void *, const void *, unsigned int);

// The member offsets the clamp reads are pinned by this layout and by nothing
// else: `next` at 0x1c and `end_read` at 0x20 are what `_MMIOINFO` has, and the
// two must be DISTINCT — equal offsets are a compare of a value with itself,
// which c2 folds into a body this class has not been graded on.
struct wifn_info {
    unsigned pad[7]; // 0x00 .. 0x1b
    char *next;      // 0x1c
    char *end_read;  // 0x20
    unsigned tail[9];
};

// ---- sub-shape G: dst is formal 1, src is formal 0 -------------------------
extern "C" long wifn_get_info(void *h, wifn_info *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 0x48);
    return 0;
}

// ---- sub-shape S: dst is formal 0, src is formal 1, then the clamp ---------
extern "C" long wifn_set_info(void *h, wifn_info *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(h, p, 0x48);
    wifn_info *ni = (wifn_info *)h;
    if (ni->end_read < ni->next) {
        ni->end_read = ni->next;
    }
    return 0;
}
