// w-mmio park grid cell mid_n5_p04132_g2_ln
// arity 5 perm [0, 4, 1, 3, 2] guards [2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    g5(a0, a4, a1, a3, a2);
    return 0;
}
