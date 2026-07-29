// IL statement-grammar characterization: `if`/`else`.
//
// The same condition and then-clause as il_stmt_if.cpp with an else added, so
// the byte delta is exactly the else arm: the extra unconditional branch over it
// and the extra label.
//
// See docs/IL_STMT_GRAMMAR.md §7.

extern void g();
extern void h();

void stmt_if_else(int a) {
    if (a) g();
    else h();
}

void stmt_if_else_brace(int a) {
    if (a) { g(); }
    else { h(); }
}
