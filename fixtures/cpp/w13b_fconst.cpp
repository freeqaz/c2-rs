// W13b — float constants. Each distinct constant costs an `.rdata` COMDAT
// section, a section-symbol/external-symbol pair named `__real@<ieee-hex>`, and
// four relocations at the reference site (REFHI+PAIR on the `addis`, REFLO+PAIR
// on the `lfs`). Dedup is by bit pattern, TU-wide.
//
// Kept to ONE constant and one function so the obj is the minimal witness of the
// shape; the multi-constant and dedup cases live in w13_fneg.cpp.

float k_add(float a) { return a + 1.0f; }
