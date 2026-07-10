// Minimal freestanding int fixtures (include-free): a handful of tiny pure
// functions whose IL bundles are trivial to capture and whose obj is small
// and deterministic. Used by the oracle self-test (determinism + capture
// stability). No headers — the reference toolchain compiles these with
// /Ox /GS- /c and nothing else.

int add3(int a, int b, int c) {
    return a + b + c;
}

int mul_add(int a, int b, int c) {
    return a * b + c;
}

int select_max(int a, int b) {
    return a > b ? a : b;
}

int shift_mask(int x, int n) {
    return (x << n) & 0xff;
}
