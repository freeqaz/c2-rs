// w-mmio park grid cell g2ord_n4_p3021_g13_ln
// arity 4 perm [3, 0, 2, 1] guards [1, 3] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    if (a3 == 0) return 11;
    g4(a3, a0, a2, a1);
    return 0;
}
