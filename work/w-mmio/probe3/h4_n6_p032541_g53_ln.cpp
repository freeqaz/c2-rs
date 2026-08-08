// w-mmio park grid cell h4_n6_p032541_g53_ln
// arity 6 perm [0, 3, 2, 5, 4, 1] guards [5, 3] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a5 == 0) return 5;
    if (a3 == 0) return 11;
    g6(a0, a3, a2, a5, a4, a1);
    return 0;
}
