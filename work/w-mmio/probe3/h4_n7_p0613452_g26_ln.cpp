// w-mmio park grid cell h4_n7_p0613452_g26_ln
// arity 7 perm [0, 6, 1, 3, 4, 5, 2] guards [2, 6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a2 == 0) return 5;
    if (a6 == 0) return 11;
    g7(a0, a6, a1, a3, a4, a5, a2);
    return 0;
}
