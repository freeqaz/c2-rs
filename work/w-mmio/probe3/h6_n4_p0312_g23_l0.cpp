// w-mmio park grid cell h6_n4_p0312_g23_l0
// arity 4 perm [0, 3, 1, 2] guards [2, 3] calls 1 lit 0
void g4(unsigned, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    g4(72, a3, a1, a2);
    return 0;
}
