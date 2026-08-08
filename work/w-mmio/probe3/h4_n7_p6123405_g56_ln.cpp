// w-mmio park grid cell h4_n7_p6123405_g56_ln
// arity 7 perm [6, 1, 2, 3, 4, 0, 5] guards [5, 6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a5 == 0) return 5;
    if (a6 == 0) return 11;
    g7(a6, a1, a2, a3, a4, a0, a5);
    return 0;
}
