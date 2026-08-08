// w-mmio park grid cell gtgt_n4_p1203_g2_c1_ln
// arity 4 perm [1, 2, 0, 3] guards [2] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    g4(a1, a2, a0, a3);
    return 0;
}
