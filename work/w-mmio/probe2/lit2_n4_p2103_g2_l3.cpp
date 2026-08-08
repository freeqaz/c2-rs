// w-mmio park grid cell lit2_n4_p2103_g2_l3
// arity 4 perm [2, 1, 0, 3] guards [2] calls 1 lit 3
void g4(void *, void *, void *, unsigned);
int f(void *a0, void *a1, void *a2, void *a3) {
    if (a2 == 0) return 5;
    g4(a2, a1, a0, 72);
    return 0;
}
