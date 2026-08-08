// w-mmio park grid cell lit_n4_p0132_g2_c1_l1
// arity 4 perm [0, 1, 3, 2] guards [2] calls 1 lit 1
void g4(void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    g4(a0, 72, a3, a2);
    return 0;
}
