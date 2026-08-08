// w-mmio park grid cell wide_n6_p510342_g0_ln
// arity 6 perm [5, 1, 0, 3, 4, 2] guards [0] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a0 == 0) return 5;
    g6(a5, a1, a0, a3, a4, a2);
    return 0;
}
