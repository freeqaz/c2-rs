// w-mmio park grid cell h6_n4_p2130_g32_l1
// arity 4 perm [2, 1, 3, 0] guards [3, 2] calls 1 lit 1
void g4(void *, unsigned, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a2 == 0) return 11;
    g4(a2, 72, a3, a0);
    return 0;
}
