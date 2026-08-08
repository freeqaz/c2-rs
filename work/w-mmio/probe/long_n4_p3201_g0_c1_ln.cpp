// w-mmio park grid cell long_n4_p3201_g0_c1_ln
// arity 4 perm [3, 2, 0, 1] guards [0] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    g4(a3, a2, a0, a1);
    return 0;
}
