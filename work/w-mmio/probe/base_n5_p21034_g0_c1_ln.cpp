// w-mmio park grid cell base_n5_p21034_g0_c1_ln
// arity 5 perm [2, 1, 0, 3, 4] guards [0] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a0 == 0) return 5;
    g5(a2, a1, a0, a3, a4);
    return 0;
}
