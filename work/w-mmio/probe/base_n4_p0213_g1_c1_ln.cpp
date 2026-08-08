// w-mmio park grid cell base_n4_p0213_g1_c1_ln
// arity 4 perm [0, 2, 1, 3] guards [1] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    g4(a0, a2, a1, a3);
    return 0;
}
