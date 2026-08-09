// w-xtea2 probe — the SWAP cell, isolating the register-plan clause from every
// other one. Two plain pointer formals, no member offsets, a length inside the
// call window: the ONLY thing that differs from the accepted class is which
// formal is the destination.
extern "C" void *memcpy(void *, const void *, unsigned long);

// accepted: dst is formal 0, src is formal 1 — both already in place
void ok2(unsigned char *d, const unsigned char *s) { memcpy(d, s, 0x10); }

// the cell: dst is formal 1, src is formal 0 — the two must be exchanged
void swap2(const unsigned char *s, unsigned char *d) { memcpy(d, s, 0x10); }
