// w-mmio park grid cell base_n5_p03241_g1_c1_ln
// arity 5 perm [0, 3, 2, 4, 1] guards [1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    g5(a0, a3, a2, a4, a1);
    return 0;
}
