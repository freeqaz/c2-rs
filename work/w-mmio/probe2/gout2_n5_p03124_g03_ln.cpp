// w-mmio park grid cell gout2_n5_p03124_g03_ln
// arity 5 perm [0, 3, 1, 2, 4] guards [0, 3] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a0 == 0) return 5;
    if (a3 == 0) return 11;
    g5(a0, a3, a1, a2, a4);
    return 0;
}
