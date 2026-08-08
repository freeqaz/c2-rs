// w-mmio park grid cell h6_n4_p3021_g13_l2
// arity 4 perm [3, 0, 2, 1] guards [1, 3] calls 1 lit 2
void g4(void *, void *, unsigned, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    if (a3 == 0) return 11;
    g4(a3, a0, 72, a1);
    return 0;
}
