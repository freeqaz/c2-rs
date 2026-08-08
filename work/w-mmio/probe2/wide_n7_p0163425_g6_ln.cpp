// w-mmio park grid cell wide_n7_p0163425_g6_ln
// arity 7 perm [0, 1, 6, 3, 4, 2, 5] guards [6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a6 == 0) return 5;
    g7(a0, a1, a6, a3, a4, a2, a5);
    return 0;
}
