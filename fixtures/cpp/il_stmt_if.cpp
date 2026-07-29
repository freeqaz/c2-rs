// IL statement-grammar characterization: `if` with NO else.
//
// Braced against unbraced then-clause. `53` opens the function body and also
// nested scopes, so the pair separates "the `if` statement itself opens a scope"
// from "the braced block opens it": an unbraced then-clause has one fewer brace
// in the source and so must have one fewer scope marker if `53` tracks braces.
//
// See docs/IL_STMT_GRAMMAR.md §7.

extern void g();

void stmt_if_nobrace(int a) {
    if (a) g();
}

void stmt_if_brace(int a) {
    if (a) { g(); }
}

void stmt_if_cmp(int a) {
    if (a > 0) g();
}
