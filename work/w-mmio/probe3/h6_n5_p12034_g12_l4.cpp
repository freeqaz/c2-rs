// w-mmio park grid cell h6_n5_p12034_g12_l4
// arity 5 perm [1, 2, 0, 3, 4] guards [1, 2] calls 1 lit 4
void g5(void *, void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a2 == 0) return 11;
    g5(a1, a2, a0, a3, 72);
    return 0;
}
