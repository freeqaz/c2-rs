// IL statement-grammar characterization: a SEQUENCE of expression statements.
//
// A statement-count ladder in one TU. Everything else is held fixed (same
// callee, same void signature, no locals, no control flow), so the byte delta
// between consecutive bodies is exactly "one more expression statement".
// Answers: what separates two statements, and is the separator emitted after
// the last one too.
//
// See docs/IL_STMT_GRAMMAR.md §2.

extern void g();

void stmt_seq0() {}
void stmt_seq1() { g(); }
void stmt_seq2() { g(); g(); }
void stmt_seq3() { g(); g(); g(); }
