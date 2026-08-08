// w-mmio park grid cell lit_n4_p0213_g1_c1_l3
// arity 4 perm [0, 2, 1, 3] guards [1] calls 1 lit 3
void g4(void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    g4(a0, a2, a1, 72);
    return 0;
}
