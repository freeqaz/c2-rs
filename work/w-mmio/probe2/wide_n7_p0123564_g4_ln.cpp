// w-mmio park grid cell wide_n7_p0123564_g4_ln
// arity 7 perm [0, 1, 2, 3, 5, 6, 4] guards [4] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a4 == 0) return 5;
    g7(a0, a1, a2, a3, a5, a6, a4);
    return 0;
}
