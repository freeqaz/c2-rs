// w-mmio park grid cell base_n3_p120_g0_c1_ln
// arity 3 perm [1, 2, 0] guards [0] calls 1 lit None
void g3(void *, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a1, a2, a0);
    return 0;
}
