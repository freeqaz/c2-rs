// w-mmio park grid cell wide_n6_p312540_g5_ln
// arity 6 perm [3, 1, 2, 5, 4, 0] guards [5] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a5 == 0) return 5;
    g6(a3, a1, a2, a5, a4, a0);
    return 0;
}
