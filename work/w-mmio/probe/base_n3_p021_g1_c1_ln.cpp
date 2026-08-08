// w-mmio park grid cell base_n3_p021_g1_c1_ln
// arity 3 perm [0, 2, 1] guards [1] calls 1 lit None
void g3(void *, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a1 == 0) return 5;
    g3(a0, a2, a1);
    return 0;
}
