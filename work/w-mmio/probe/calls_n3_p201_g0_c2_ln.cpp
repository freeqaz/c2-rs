// w-mmio park grid cell calls_n3_p201_g0_c2_ln
// arity 3 perm [2, 0, 1] guards [0] calls 2 lit None
void g3(void *, void *, void *);
void h(void *);
int f(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    g3(a2, a0, a1);
    h(a2);
    return 0;
}
