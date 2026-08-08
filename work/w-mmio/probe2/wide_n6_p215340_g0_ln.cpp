// w-mmio park grid cell wide_n6_p215340_g0_ln
// arity 6 perm [2, 1, 5, 3, 4, 0] guards [0] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a0 == 0) return 5;
    g6(a2, a1, a5, a3, a4, a0);
    return 0;
}
