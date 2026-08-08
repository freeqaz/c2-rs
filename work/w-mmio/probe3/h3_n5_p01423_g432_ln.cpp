// w-mmio park grid cell h3_n5_p01423_g432_ln
// arity 5 perm [0, 1, 4, 2, 3] guards [4, 3, 2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a4 == 0) return 5;
    if (a3 == 0) return 11;
    if (a2 == 0) return 7;
    g5(a0, a1, a4, a2, a3);
    return 0;
}
