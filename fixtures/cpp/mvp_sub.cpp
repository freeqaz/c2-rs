// Non-commutative int fixture: subtraction (subf, reversed operands) mixed with
// add and multiply. Pins the load-bearing operand order for subf — a swap would
// be a fuzzy-invisible sign inversion (see codegen::encode_subf, CLAUDE.md
// correctness boundary). Straight-line, no branches/relocs.
int sub3(int a, int b, int c) {
    return a - b - c;
}

int mul3(int a, int b, int c) {
    return a * b * c;
}

int submix(int a, int b, int c) {
    return a - b + c;
}
