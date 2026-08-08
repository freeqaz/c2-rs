// w-mmio park grid cell h6_n4_p2013_g21_l3
// arity 4 perm [2, 0, 1, 3] guards [2, 1] calls 1 lit 3
void g4(void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a1 == 0) return 11;
    g4(a2, a0, a1, 72);
    return 0;
}
