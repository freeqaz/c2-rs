// Multi-function straight-line int fixture: two pure add-chain functions in one
// TU. Exercises PortC2's multi-function COFF path — several .text function
// symbols with cumulative Value offsets, packed contiguously — while staying
// inside the commutative-add codegen class (no branches/relocs). See
// docs/OBJ_FORMAT_MVP.md and the differential test.
int add2(int a, int b) {
    return a + b;
}

int add4(int a, int b, int c, int d) {
    return a + b + c + d;
}
