// w-mmio park grid cell h5_n5_p01423_g01_ln
// arity 5 perm [0, 1, 4, 2, 3] guards [0, 1] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g5(a0, a1, a4, a2, a3);
    return 0;
}
