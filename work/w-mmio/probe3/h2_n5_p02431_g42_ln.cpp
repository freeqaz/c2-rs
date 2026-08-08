// w-mmio park grid cell h2_n5_p02431_g42_ln
// arity 5 perm [0, 2, 4, 3, 1] guards [4, 2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a2 == 0) return 11;
    g5(a0, a2, a4, a3, a1);
    return 0;
}
