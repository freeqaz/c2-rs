// w-mmio park grid cell wide_n7_p6123054_g0_ln
// arity 7 perm [6, 1, 2, 3, 0, 5, 4] guards [0] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a0 == 0) return 5;
    g7(a6, a1, a2, a3, a0, a5, a4);
    return 0;
}
