// w-mmio park grid cell gout_n5_p21304_g1_c1_ln
// arity 5 perm [2, 1, 3, 0, 4] guards [1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    g5(a2, a1, a3, a0, a4);
    return 0;
}
