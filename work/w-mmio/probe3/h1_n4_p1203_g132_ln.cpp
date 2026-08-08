// w-mmio park grid cell h1_n4_p1203_g132_ln
// arity 4 perm [1, 2, 0, 3] guards [1, 3, 2] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a1 == 0) return 5;
    if (a3 == 0) return 11;
    if (a2 == 0) return 7;
    g4(a1, a2, a0, a3);
    return 0;
}
