// w-mmio park grid cell h6_n5_p41203_g43_l2
// arity 5 perm [4, 1, 2, 0, 3] guards [4, 3] calls 1 lit 2
void g5(void *, void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a3 == 0) return 11;
    g5(a4, a1, 72, a0, a3);
    return 0;
}
