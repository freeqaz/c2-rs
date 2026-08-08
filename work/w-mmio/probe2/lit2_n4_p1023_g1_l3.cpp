// w-mmio park grid cell lit2_n4_p1023_g1_l3
// arity 4 perm [1, 0, 2, 3] guards [1] calls 1 lit 3
void g4(void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    g4(a1, a0, a2, 72);
    return 0;
}
