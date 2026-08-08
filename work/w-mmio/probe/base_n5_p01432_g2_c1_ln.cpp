// w-mmio park grid cell base_n5_p01432_g2_c1_ln
// arity 5 perm [0, 1, 4, 3, 2] guards [2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    g5(a0, a1, a4, a3, a2);
    return 0;
}
