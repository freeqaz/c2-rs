// w-mmio park grid cell wide_n6_p015243_g2_ln
// arity 6 perm [0, 1, 5, 2, 4, 3] guards [2] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a2 == 0) return 5;
    g6(a0, a1, a5, a2, a4, a3);
    return 0;
}
