// w-mmio park grid cell h2_n5_p14230_g41_ln
// arity 5 perm [1, 4, 2, 3, 0] guards [4, 1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a1 == 0) return 11;
    g5(a1, a4, a2, a3, a0);
    return 0;
}
