// w-mmio park grid cell h3_n5_p30214_g130_ln
// arity 5 perm [3, 0, 2, 1, 4] guards [1, 3, 0] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a1 == 0) return 5;
    if (a3 == 0) return 11;
    if (a0 == 0) return 7;
    g5(a3, a0, a2, a1, a4);
    return 0;
}
