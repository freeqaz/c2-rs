// w-mmio park grid cell h6_n4_p0231_g32_l0
// arity 4 perm [0, 2, 3, 1] guards [3, 2] calls 1 lit 0
void g4(unsigned, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a2 == 0) return 11;
    g4(72, a2, a3, a1);
    return 0;
}
