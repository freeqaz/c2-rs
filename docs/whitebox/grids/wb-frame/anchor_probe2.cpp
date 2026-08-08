// wb-frame AUXILIARY probe 2 — pins the mechanism behind the ?supershuffle
// anchor's real defect. Declared separately from the frozen frame grid.
//
// The callees are loops over a runtime bound so c2 will not inline them; that
// isolates the register question from the inlining question, which probe 1
// could not do. The two arms differ ONLY in definition order:
//
//   arm C: callees defined BEFORE the caller (?supershuffle's own order)
//   arm D: callees defined AFTER the caller
//
// If c2 keeps the pointer in the volatile r3 across the calls in arm C and not
// in arm D, the mechanism is "the callee was already compiled, so its register
// footprint is known" and it is order-sensitive. If both arms keep r3, the
// analysis is whole-TU and order-free. If neither does, probe 1's reading of
// the anchor is wrong and this lane says so.

extern volatile int wbfc_n;

// ---- arm C: callees first --------------------------------------------------
static void wbfc_leaf1(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] + 1);
}
static void wbfc_leaf2(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] ^ 2);
}
static void wbfc_leaf3(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] - 3);
}

void wbfc_callee_first(char *c) {
    wbfc_leaf1(c);
    wbfc_leaf2(c);
    wbfc_leaf3(c);
}

// ---- arm D: caller first ---------------------------------------------------
static void wbfd_leaf1(char *c);
static void wbfd_leaf2(char *c);
static void wbfd_leaf3(char *c);

void wbfd_caller_first(char *c) {
    wbfd_leaf1(c);
    wbfd_leaf2(c);
    wbfd_leaf3(c);
}

static void wbfd_leaf1(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] + 1);
}
static void wbfd_leaf2(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] ^ 2);
}
static void wbfd_leaf3(char *c) {
    int n = wbfc_n;
    for (int i = 0; i < n; i++) c[i] = (char)(c[i] - 3);
}
