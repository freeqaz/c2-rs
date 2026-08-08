// w-mmio park grid cell h6_n5_p04213_g34_l2
// arity 5 perm [0, 4, 2, 1, 3] guards [3, 4] calls 1 lit 2
void g5(void *, void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a0, a4, 72, a1, a3);
    return 0;
}
