// w-mmio park grid cell lit_n3_p102_g0_c1_l2
// arity 3 perm [1, 0, 2] guards [0] calls 1 lit 2
void g3(void *, void *, unsigned);
int f(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a1, a0, 72);
    return 0;
}
