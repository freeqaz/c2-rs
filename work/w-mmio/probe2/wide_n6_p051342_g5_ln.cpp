// w-mmio park grid cell wide_n6_p051342_g5_ln
// arity 6 perm [0, 5, 1, 3, 4, 2] guards [5] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a5 == 0) return 5;
    g6(a0, a5, a1, a3, a4, a2);
    return 0;
}
