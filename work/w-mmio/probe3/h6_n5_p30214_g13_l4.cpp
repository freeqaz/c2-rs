// w-mmio park grid cell h6_n5_p30214_g13_l4
// arity 5 perm [3, 0, 2, 1, 4] guards [1, 3] calls 1 lit 4
void g5(void *, void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a3 == 0) return 11;
    g5(a3, a0, a2, a1, 72);
    return 0;
}
