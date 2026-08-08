// w-mmio park grid cell lit_n3_p210_g0_c1_l1
// arity 3 perm [2, 1, 0] guards [0] calls 1 lit 1
void g3(void *, unsigned, void *);
int f(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a2, 72, a0);
    return 0;
}
