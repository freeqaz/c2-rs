// w-mmio park grid cell gcnt_n4_p1203_g01_c1_ln
// arity 4 perm [1, 2, 0, 3] guards [0, 1] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g4(a1, a2, a0, a3);
    return 0;
}
