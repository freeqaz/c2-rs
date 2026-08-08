// w-mmio park grid cell h4_n6_p042351_g45_ln
// arity 6 perm [0, 4, 2, 3, 5, 1] guards [4, 5] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a4 == 0) return 5;
    if (a5 == 0) return 11;
    g6(a0, a4, a2, a3, a5, a1);
    return 0;
}
