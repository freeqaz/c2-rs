// w-mmio park grid cell wide_n7_p0126435_g6_ln
// arity 7 perm [0, 1, 2, 6, 4, 3, 5] guards [6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a6 == 0) return 5;
    g7(a0, a1, a2, a6, a4, a3, a5);
    return 0;
}
