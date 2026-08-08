// w-mmio park grid cell h2_n4_p3102_g32_ln
// arity 4 perm [3, 1, 0, 2] guards [3, 2] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a2 == 0) return 11;
    g4(a3, a1, a0, a2);
    return 0;
}
