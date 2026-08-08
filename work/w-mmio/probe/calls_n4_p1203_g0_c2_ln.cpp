// w-mmio park grid cell calls_n4_p1203_g0_c2_ln
// arity 4 perm [1, 2, 0, 3] guards [0] calls 2 lit None
void g4(void *, void *, void *, void *);
void h(void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    g4(a1, a2, a0, a3);
    h(a3);
    return 0;
}
