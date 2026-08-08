// w-mmio park grid cell h4_n7_p0263451_g26_ln
// arity 7 perm [0, 2, 6, 3, 4, 5, 1] guards [2, 6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a2 == 0) return 5;
    if (a6 == 0) return 11;
    g7(a0, a2, a6, a3, a4, a5, a1);
    return 0;
}
