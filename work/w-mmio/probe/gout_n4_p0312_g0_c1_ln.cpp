// w-mmio park grid cell gout_n4_p0312_g0_c1_ln
// arity 4 perm [0, 3, 1, 2] guards [0] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    g4(a0, a3, a1, a2);
    return 0;
}
