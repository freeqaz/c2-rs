// w-mmio park grid cell gout_n4_p2013_g3_c1_ln
// arity 4 perm [2, 0, 1, 3] guards [3] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    g4(a2, a0, a1, a3);
    return 0;
}
