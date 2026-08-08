// w-mmio park grid cell long_n5_p42013_g0_c1_ln
// arity 5 perm [4, 2, 0, 1, 3] guards [0] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a0 == 0) return 5;
    g5(a4, a2, a0, a1, a3);
    return 0;
}
