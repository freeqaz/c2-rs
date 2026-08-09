// W-XTEA2 `_neg` — the copy's two operands NOT already in the registers
// `memcpy` wants: the destination is formal 1 and the source is formal 0.
//
// ONE cell per file, and that is not tidiness: a `_neg` fixture holding several
// refusing bodies can NEVER go `mismatch`, because a TU verdict is a
// CONJUNCTION over its functions — the first draft of this cell set was one
// four-body file and every must-fail mutation came back `vocab-gap`, proving
// nothing. Each cell gets its own TU so that deleting its clause makes the whole
// TU in-class and the obj is then graded byte for byte by real `c2.dll`.
//
// **And this cell is the one that had to be re-derived**, which is worth saying
// where it happened. Its first spelling was `memcpy(out, p->b, 0x10)` — the
// direction reversed AND a member offset on the source AND a receiver in play —
// so it was fenced by THREE clauses at once and deleting any one of them left
// the other two refusing it. `work/w-xtea2/probe/mcpyswap.cpp` isolates the
// register plan and nothing else:
//
//   ok2(d, s)    memcpy(d, s, 16)   li r5,16 . b memcpy                    8 B
//   swap2(s, d)  memcpy(d, s, 16)   mr r11,r4 . mr r4,r3 . li r5,16
//                                   . mr r3,r11 . b memcpy                20 B
//
// Two plain pointer formals, no member offsets, a length inside the call window,
// and c2 keeps the call — so the cell is neither vacuous nor over-fenced.
//
// THE CLAUSE: `mcpytail-operands-are-not-already-in-the-argument-registers`
// THE MUTATION: delete that clause
//   -> the port emits ONE word where c2 emits FIVE and never exchanges the
//      registers, so the copy runs backwards in an obj that links.
//
// The framed `?wxn_after` is LAST so the cell is upstream of the TU's only
// `$M`/`$M`/`$T` triple: `LABEL_COUNTER.md` §7.6 step 5 and board #2305 — a
// wrong charge on the last function of a TU moves nothing.

extern "C" void *memcpy(void *, const void *, unsigned long);

void wxn_swap(const unsigned char *s, unsigned char *d) { memcpy(d, s, 0x10); }

int wxn_gz(int);
int wxn_after(int a) { return wxn_gz(a) + 3; }
