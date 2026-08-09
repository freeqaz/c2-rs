// W-XTEA2 `_neg` — a SECOND statement after the copy — the body no longer ends at the call.
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
// verified not to be VACUOUS (c2 keeps the call; what changes is that it becomes a `bl` inside a frame).
//
// Real `c2` at `/O1 /Oi /GS- /c`:
//
//   60 B   FRAMED: mflr . stw . std r31 . stwu . mr r31,r3 . addi r3,r3,16
//          . li r5,16 . bl memcpy . li r11,0 . std r11,32(r31) . epilogue
//
// THE CLAUSE: `the `eat_return_plumbing` gate after the call's `4B``
// THE MUTATION: make the plumbing non-fatal (`let _ = ...`)
//   -> the port emits a 12-byte LEAF tail branch for a 60-byte FRAMED body, drops the store, and emits no `.pdata` record
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

void wxn_then_store(wxn_p *p, const unsigned char *s) {
    memcpy(p->b, s, 0x10);
    p->n[0] = 0;
}

int wxn_gz(int);
int wxn_after(int a) { return wxn_gz(a) + 3; }
