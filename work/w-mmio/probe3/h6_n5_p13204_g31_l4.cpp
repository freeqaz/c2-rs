// w-mmio park grid cell h6_n5_p13204_g31_l4
// arity 5 perm [1, 3, 2, 0, 4] guards [3, 1] calls 1 lit 4
void g5(void *, void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    if (a1 == 0) return 11;
    g5(a1, a3, a2, a0, 72);
    return 0;
}
