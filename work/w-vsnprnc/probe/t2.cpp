// t2 — the same forwarding leaf with NO literal: the control that says whether
// the literal is what refuses it, or the permutation.
extern "C" {
extern int callee(char *, unsigned, char *, void *, void *);
int fwd(char *b, unsigned n, char *f, void *lo, void *ap) {
    return callee(b, n, f, lo, ap);
}
}
