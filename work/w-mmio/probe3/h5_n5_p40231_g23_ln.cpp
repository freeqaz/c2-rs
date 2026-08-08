// w-mmio park grid cell h5_n5_p40231_g23_ln
// arity 5 perm [4, 0, 2, 3, 1] guards [2, 3] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    g5(a4, a0, a2, a3, a1);
    return 0;
}
