// w-mmio park grid cell hi2_n3_p102_g1_ln
// arity 3 perm [1, 0, 2] guards [1] calls 1 lit None
void g3(void *, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a1 == 0) return 5;
    g3(a1, a0, a2);
    return 0;
}
