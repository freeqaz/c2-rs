// t1 — `vsnprnc.cpp::vsprintf_s` in four lines: an UNFRAMED forwarding tail
// call, one ascending move and one literal in a middle slot.
extern "C" {
extern int callee(char *, unsigned, char *, void *, void *);
int fwd(char *b, unsigned n, char *f, void *ap) {
    return callee(b, n, f, 0, ap);
}
}
