// w-mmio park grid cell h2_n5_p40231_g41_ln
// arity 5 perm [4, 0, 2, 3, 1] guards [4, 1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a1 == 0) return 11;
    g5(a4, a0, a2, a3, a1);
    return 0;
}
