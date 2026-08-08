// w-mmio park grid cell h3_n5_p21430_g042_ln
// arity 5 perm [2, 1, 4, 3, 0] guards [0, 4, 2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a0 == 0) return 5;
    if (a4 == 0) return 11;
    if (a2 == 0) return 7;
    g5(a2, a1, a4, a3, a0);
    return 0;
}
