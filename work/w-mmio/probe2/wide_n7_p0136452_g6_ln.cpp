// w-mmio park grid cell wide_n7_p0136452_g6_ln
// arity 7 perm [0, 1, 3, 6, 4, 5, 2] guards [6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a6 == 0) return 5;
    g7(a0, a1, a3, a6, a4, a5, a2);
    return 0;
}
