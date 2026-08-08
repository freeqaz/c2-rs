// w-mmio park grid cell h4_n7_p3126450_g63_ln
// arity 7 perm [3, 1, 2, 6, 4, 5, 0] guards [6, 3] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a6 == 0) return 5;
    if (a3 == 0) return 11;
    g7(a3, a1, a2, a6, a4, a5, a0);
    return 0;
}
