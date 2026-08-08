// w-mmio park grid cell hi2_n5_p02134_g2_ln
// arity 5 perm [0, 2, 1, 3, 4] guards [2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    g5(a0, a2, a1, a3, a4);
    return 0;
}
