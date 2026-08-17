// g10 — `__declspec(selectany)`: a COMDAT initialized NON-const object by a
// different front door. Second cell for the const/.rdata discriminator.
__declspec(selectany) int sa = 3;
