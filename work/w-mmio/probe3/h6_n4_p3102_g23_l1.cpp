// w-mmio park grid cell h6_n4_p3102_g23_l1
// arity 4 perm [3, 1, 0, 2] guards [2, 3] calls 1 lit 1
void g4(void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    g4(a3, 72, a0, a2);
    return 0;
}
