// W-XTEA2 `_neg` — a copy LENGTH below the measured call window.
//
// ONE cell per file, and that is not tidiness: a `_neg` fixture holding several
// refusing bodies can NEVER go `mismatch`, because a TU verdict is a
// CONJUNCTION over its functions — the first draft of this cell set was one
// four-body file and every must-fail mutation came back `vocab-gap`, proving
// nothing. Each cell gets its own TU so that deleting its clause makes the whole
// TU in-class and the obj is then graded byte for byte by real `c2.dll`.
//
// The cell was compiled through real `c2.dll` under wibo BEFORE it was claimed
// as a refusal (`work/w-xtea2/probe/mcpyneg.cpp`), so it is verified to be a
// genuinely different body and not one the port would have got right anyway
// (`w-pool2` §5: a predecessor's `_neg` cell had become a POSITIVE), and
// verified not to be VACUOUS (the body still frames as a `40` intrinsic in the IL; only the LOWERING changes).
//
// Real `c2` at `/O1 /Oi /GS- /c`:
//
//   40 B   lbz/stb x4 — NO CALL AT ALL; c2 expands below the window
//
// THE CLAUSE: `mcpytail-length-outside-the-call-window`
// THE MUTATION: widen the window's floor from `MEMCPY_CALL_STEP` to 1
//   -> the port emits `li r5,4 . b memcpy` for a body c2 expanded inline
//
// The framed `?wxn_after` is LAST so the cell is upstream of the TU's only
// `$M`/`$M`/`$T` triple: `LABEL_COUNTER.md` §7.6 step 5 and board #2305 — a
// wrong charge on the last function of a TU moves nothing.

extern "C" void *memcpy(void *, const void *, unsigned long);

struct wxn_p {
    unsigned char a[16];     // 0x00
    unsigned char b[16];     // 0x10
    unsigned long long n[2]; // 0x20
    unsigned char big[64];   // 0x30
    unsigned char big2[64];  // 0x70
};

void wxn_short(wxn_p *p, const unsigned char *s) { memcpy(p->b, s, 0x4); }

int wxn_gz(int);
int wxn_after(int a) { return wxn_gz(a) + 3; }
