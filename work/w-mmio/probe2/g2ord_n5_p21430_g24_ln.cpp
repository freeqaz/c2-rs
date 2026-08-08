// w-mmio park grid cell g2ord_n5_p21430_g24_ln
// arity 5 perm [2, 1, 4, 3, 0] guards [2, 4] calls 1 lit None
void g5(void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4) {
    if (a2 == 0) return 5;
    if (a4 == 0) return 11;
    g5(a2, a1, a4, a3, a0);
    return 0;
}
