// t3 — the literal in a middle slot with NO move: does the permutation alone
// refuse, or the literal alone?
extern "C" {
extern int callee(char *, unsigned, char *, void *);
int fwd(char *b, unsigned n, char *f) {
    return callee(b, n, f, 0);
}
}
