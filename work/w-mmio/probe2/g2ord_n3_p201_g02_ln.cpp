// w-mmio park grid cell g2ord_n3_p201_g02_ln
// arity 3 perm [2, 0, 1] guards [0, 2] calls 1 lit None
void g3(void *, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    if (a2 == 0) return 11;
    g3(a2, a0, a1);
    return 0;
}
