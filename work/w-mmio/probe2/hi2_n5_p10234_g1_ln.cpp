// w-mmio park grid cell hi2_n5_p10234_g1_ln
// arity 5 perm [1, 0, 2, 3, 4] guards [1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    g5(a1, a0, a2, a3, a4);
    return 0;
}
