// w-mmio park grid cell lit2_n4_p3120_g3_l2
// arity 4 perm [3, 1, 2, 0] guards [3] calls 1 lit 2
void g4(void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    g4(a3, a1, 72, a0);
    return 0;
}
