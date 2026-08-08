// w-mmio park grid cell h3_n4_p0231_g312_ln
// arity 4 perm [0, 2, 3, 1] guards [3, 1, 2] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a3 == 0) return 5;
    if (a1 == 0) return 11;
    if (a2 == 0) return 7;
    g4(a0, a2, a3, a1);
    return 0;
}
