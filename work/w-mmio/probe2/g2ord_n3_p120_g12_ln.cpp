// w-mmio park grid cell g2ord_n3_p120_g12_ln
// arity 3 perm [1, 2, 0] guards [1, 2] calls 1 lit None
void g3(void *, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a1 == 0) return 5;
    if (a2 == 0) return 11;
    g3(a1, a2, a0);
    return 0;
}
