// w-mmio park grid cell lit2_n3_p021_g2_l0
// arity 3 perm [0, 2, 1] guards [2] calls 1 lit 0
void g3(unsigned, void *, void *);
int f(void *a0, void *a1, void *a2) {
    if (a2 == 0) return 5;
    g3(72, a2, a1);
    return 0;
}
