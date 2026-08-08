// w-mmio park grid cell wide_n7_p0623415_g1_ln
// arity 7 perm [0, 6, 2, 3, 4, 1, 5] guards [1] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a1 == 0) return 5;
    g7(a0, a6, a2, a3, a4, a1, a5);
    return 0;
}
