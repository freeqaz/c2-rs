// w-mmio park grid cell g2ord_n4_p1320_g30_ln
// arity 4 perm [1, 3, 2, 0] guards [3, 0] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a0 == 0) return 11;
    g4(a1, a3, a2, a0);
    return 0;
}
