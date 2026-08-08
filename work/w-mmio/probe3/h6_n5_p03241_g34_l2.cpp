// w-mmio park grid cell h6_n5_p03241_g34_l2
// arity 5 perm [0, 3, 2, 4, 1] guards [3, 4] calls 1 lit 2
void g5(void *, void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a0, a3, 72, a4, a1);
    return 0;
}
