// w-mmio park grid cell h6_n5_p04132_g42_l3
// arity 5 perm [0, 4, 1, 3, 2] guards [4, 2] calls 1 lit 3
void g5(void *, void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a2 == 0) return 11;
    g5(a0, a4, a1, 72, a2);
    return 0;
}
