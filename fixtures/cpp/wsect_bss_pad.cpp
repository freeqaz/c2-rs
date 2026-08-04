// w-sect / board #174 — two `.bss` objects WITH alignment padding, the other
// side of that axis. `char` is align 1 and `double` is align 8, so the bump
// rounds 1 -> 8 and the section is 16 bytes. Rule A3' has no free list, so the
// [1,8) gap is never reused; §5.4's refuted hole-reuse allocator would.
// The section nibble is the MAX over the objects (Rule B1) — ALIGN_8, not
// ALIGN_1.
char c1;
double d1;
