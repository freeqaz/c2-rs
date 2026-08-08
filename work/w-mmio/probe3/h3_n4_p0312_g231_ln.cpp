// w-mmio park grid cell h3_n4_p0312_g231_ln
// arity 4 perm [0, 3, 1, 2] guards [2, 3, 1] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a3 == 0) return 11;
    if (a1 == 0) return 7;
    g4(a0, a3, a1, a2);
    return 0;
}
