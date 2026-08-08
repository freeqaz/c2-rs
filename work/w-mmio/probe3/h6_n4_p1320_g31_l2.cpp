// w-mmio park grid cell h6_n4_p1320_g31_l2
// arity 4 perm [1, 3, 2, 0] guards [3, 1] calls 1 lit 2
void g4(void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a1 == 0) return 11;
    g4(a1, a3, 72, a0);
    return 0;
}
