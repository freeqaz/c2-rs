// w-mmio park grid cell hi2_n5_p01432_g4_ln
// arity 5 perm [0, 1, 4, 3, 2] guards [4] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    g5(a0, a1, a4, a3, a2);
    return 0;
}
