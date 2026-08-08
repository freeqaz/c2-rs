// w-mmio park grid cell h4_n7_p4123650_g46_ln
// arity 7 perm [4, 1, 2, 3, 6, 5, 0] guards [4, 6] calls 1 lit None
void g7(void *, void *, void *, void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5, void *a6) {
    if (a4 == 0) return 5;
    if (a6 == 0) return 11;
    g7(a4, a1, a2, a3, a6, a5, a0);
    return 0;
}
