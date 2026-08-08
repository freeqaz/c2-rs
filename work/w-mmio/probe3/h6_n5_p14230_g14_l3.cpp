// w-mmio park grid cell h6_n5_p14230_g14_l3
// arity 5 perm [1, 4, 2, 3, 0] guards [1, 4] calls 1 lit 3
void g5(void *, void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a1, a4, a2, 72, a0);
    return 0;
}
