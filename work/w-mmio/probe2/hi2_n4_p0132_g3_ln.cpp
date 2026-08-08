// w-mmio park grid cell hi2_n4_p0132_g3_ln
// arity 4 perm [0, 1, 3, 2] guards [3] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    g4(a0, a1, a3, a2);
    return 0;
}
