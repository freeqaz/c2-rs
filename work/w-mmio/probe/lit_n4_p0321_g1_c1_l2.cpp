// w-mmio park grid cell lit_n4_p0321_g1_c1_l2
// arity 4 perm [0, 3, 2, 1] guards [1] calls 1 lit 2
void g4(void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    g4(a0, a3, 72, a1);
    return 0;
}
