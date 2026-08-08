// w-mmio park grid cell base_n2_p10_g0_c1_ln
// arity 2 perm [1, 0] guards [0] calls 1 lit None
void g2(void *, void *);
int f(void *a0, void *a1) {
    if (a0 == 0) return 5;
    g2(a1, a0);
    return 0;
}
