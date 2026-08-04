// w-sect / board #174 — MUST REFUSE. Three objects in one non-COMDAT section
// is 38 of 62 (§8.1) and the residual is walk order, board #184.
// This is the fixture on the far side of the class bound: it differs from
// wsect_bss_two.cpp in the object COUNT and in nothing else.
int b1;
int b2;
int b3;
