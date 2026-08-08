// w-mmio park grid cell base_n4_p3120_g0_c1_ln
// arity 4 perm [3, 1, 2, 0] guards [0] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    g4(a3, a1, a2, a0);
    return 0;
}
