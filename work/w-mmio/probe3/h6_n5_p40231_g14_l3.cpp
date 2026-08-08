// w-mmio park grid cell h6_n5_p40231_g14_l3
// arity 5 perm [4, 0, 2, 3, 1] guards [1, 4] calls 1 lit 3
void g5(void *, void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a4, a0, a2, 72, a1);
    return 0;
}
