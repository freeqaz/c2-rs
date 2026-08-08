// w-mmio park grid cell wide_n7_p0523461_g1_ln
// arity 7 perm [0, 5, 2, 3, 4, 6, 1] guards [1] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a1 == 0) return 5;
    g7(a0, a5, a2, a3, a4, a6, a1);
    return 0;
}
