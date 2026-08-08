// w-mmio park grid cell h4_n7_p0124653_g64_ln
// arity 7 perm [0, 1, 2, 4, 6, 5, 3] guards [6, 4] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a6 == 0) return 5;
    if (a4 == 0) return 11;
    g7(a0, a1, a2, a4, a6, a5, a3);
    return 0;
}
