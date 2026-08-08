// w-mmio park grid cell gtgt_n5_p02314_g3_c1_ln
// arity 5 perm [0, 2, 3, 1, 4] guards [3] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    g5(a0, a2, a3, a1, a4);
    return 0;
}
