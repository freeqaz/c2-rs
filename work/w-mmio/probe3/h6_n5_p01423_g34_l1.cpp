// w-mmio park grid cell h6_n5_p01423_g34_l1
// arity 5 perm [0, 1, 4, 2, 3] guards [3, 4] calls 1 lit 1
void g5(void *, unsigned, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a0, 72, a4, a2, a3);
    return 0;
}
