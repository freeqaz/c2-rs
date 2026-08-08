// w-mmio park grid cell h3_n4_p2013_g201_ln
// arity 4 perm [2, 0, 1, 3] guards [2, 0, 1] calls 1 lit None
void g4(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    if (a0 == 0) return 11;
    if (a1 == 0) return 7;
    g4(a2, a0, a1, a3);
    return 0;
}
