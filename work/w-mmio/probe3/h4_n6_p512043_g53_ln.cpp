// w-mmio park grid cell h4_n6_p512043_g53_ln
// arity 6 perm [5, 1, 2, 0, 4, 3] guards [5, 3] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a5 == 0) return 5;
    if (a3 == 0) return 11;
    g6(a5, a1, a2, a0, a4, a3);
    return 0;
}
