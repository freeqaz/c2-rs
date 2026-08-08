// w-mmio park grid cell h3_n5_p41203_g340_ln
// arity 5 perm [4, 1, 2, 0, 3] guards [3, 4, 0] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a3 == 0) return 5;
    if (a4 == 0) return 11;
    if (a0 == 0) return 7;
    g5(a4, a1, a2, a0, a3);
    return 0;
}
