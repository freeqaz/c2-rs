// wb-frame AUXILIARY probe 3 — the minimal reproducer of the ?supershuffle
// mechanism. Probe 2 failed to isolate it: single-caller `static` callees are
// always inlined, so both of its arms collapsed to one frameless body.
//
// Here the callees have EXTERNAL linkage (as ?shuffle1..6 do) and loop over a
// runtime bound, so c2 emits them as their own COMDATs and does not inline
// them. The question is whether the caller then keeps the incoming pointer in
// the volatile r3 across the calls (c2's ?supershuffle behaviour, nSaved = 0)
// or parks it in r31 (the ABI behaviour, which is what the port emits).

extern volatile int wbfe_n;

void wbfe_leaf1(char *c) {
    int n = wbfe_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] + 1);
}
void wbfe_leaf2(char *c) {
    int n = wbfe_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] ^ 2);
}
void wbfe_leaf3(char *c) {
    int n = wbfe_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] - 3);
}

void wbfe_caller(char *c) {
    wbfe_leaf1(c);
    wbfe_leaf2(c);
    wbfe_leaf3(c);
}
