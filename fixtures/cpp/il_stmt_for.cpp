// IL statement-grammar characterization: `for`.
//
// A `for` is an init statement, a condition, an increment statement and a body,
// and the interesting question is the ORDER they are emitted in relative to the
// loop's labels. Braced and unbraced bodies again separate the statement's own
// scope from the block's.
//
// See docs/IL_STMT_GRAMMAR.md §8.

extern void g();

void stmt_for_brace(int n) {
    for (int i = 0; i < n; i = i + 1) { g(); }
}

void stmt_for_nobrace(int n) {
    for (int i = 0; i < n; i = i + 1) g();
}
