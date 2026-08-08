// w-mmio park grid cell h3_n5_p12034_g102_ln
// arity 5 perm [1, 2, 0, 3, 4] guards [1, 0, 2] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a0 == 0) return 11;
    if (a2 == 0) return 7;
    g5(a1, a2, a0, a3, a4);
    return 0;
}
