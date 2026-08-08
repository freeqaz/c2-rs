// w-mmio park grid cell hi2_n5_p41230_g4_ln
// arity 5 perm [4, 1, 2, 3, 0] guards [4] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    g5(a4, a1, a2, a3, a0);
    return 0;
}
