// DECLARED: {op:27, op:32} at least -- the byte-offset add for the member and
// the store. Written as a SECOND KIND of cell: its declared set is a LOWER
// bound, not an equality, because the member store's plumbing has not been
// enumerated by hand and a cell whose prediction cannot fail is not a cell.
struct S { int a; int b; };
void p6(S *s, int v) { s->b = v; }
