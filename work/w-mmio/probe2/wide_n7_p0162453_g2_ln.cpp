// w-mmio park grid cell wide_n7_p0162453_g2_ln
// arity 7 perm [0, 1, 6, 2, 4, 5, 3] guards [2] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a2 == 0) return 5;
    g7(a0, a1, a6, a2, a4, a5, a3);
    return 0;
}
