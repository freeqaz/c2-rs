// w-mmio park grid cell h4_n7_p6120453_g36_ln
// arity 7 perm [6, 1, 2, 0, 4, 5, 3] guards [3, 6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a3 == 0) return 5;
    if (a6 == 0) return 11;
    g7(a6, a1, a2, a0, a4, a5, a3);
    return 0;
}
