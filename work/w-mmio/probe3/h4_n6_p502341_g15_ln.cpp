// w-mmio park grid cell h4_n6_p502341_g15_ln
// arity 6 perm [5, 0, 2, 3, 4, 1] guards [1, 5] calls 1 lit None
void g6(void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5) {
    if (a1 == 0) return 5;
    if (a5 == 0) return 11;
    g6(a5, a0, a2, a3, a4, a1);
    return 0;
}
