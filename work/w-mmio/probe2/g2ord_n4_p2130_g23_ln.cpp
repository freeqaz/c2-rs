// w-mmio park grid cell g2ord_n4_p2130_g23_ln
// arity 4 perm [2, 1, 3, 0] guards [2, 3] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    g4(a2, a1, a3, a0);
    return 0;
}
