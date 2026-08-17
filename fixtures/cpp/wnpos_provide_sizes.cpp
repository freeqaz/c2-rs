// w-npos — the width boundary, kept as a REFUSAL fixture: sizes 1, 2, 8 in
// one TU. c2 emits three `.rdata` COMDATs in declaration order with
// alignment nibbles 1, 3, 4 (the u16 cell is ALIGN_4, not ALIGN_2 — the cell
// that separates the measured table from the natural-alignment identity).
// The port's `.in` scalar reader does not decode every one of these widths
// yet, so the recognizer refuses the TU (no value => no object => None) and
// the expected verdict is NotImplemented — never Match, never Mismatch. The
// emitter itself already carries all three nibbles; the distance is the
// reader, priced in the w-npos rung's found-and-not-taken.
__declspec(selectany) extern const unsigned char wnpos_c = 0xAA;
__declspec(selectany) extern const unsigned short wnpos_s = 0xBBCC;
__declspec(selectany) extern const unsigned long long wnpos_q = 0x1122334455667788ULL;
