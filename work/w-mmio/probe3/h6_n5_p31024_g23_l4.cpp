// w-mmio park grid cell h6_n5_p31024_g23_l4
// arity 5 perm [3, 1, 0, 2, 4] guards [2, 3] calls 1 lit 4
void g5(void *, void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    g5(a3, a1, a0, a2, 72);
    return 0;
}
