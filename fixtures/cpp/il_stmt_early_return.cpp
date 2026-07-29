// IL statement-grammar characterization: a `return` in the MIDDLE of a body.
//
// Every accepted body so far ends in the one return-plumbing tail. A non-final
// return must therefore be something else — the plumbing's `3A <tok>` store plus
// a jump to the shared epilogue — and these bodies show what. The int and void
// cases are both present because the int one has a value to store and the void
// one does not.
//
// See docs/IL_STMT_GRAMMAR.md §9.

extern void g();

int stmt_early_int(int a) {
    if (a) return 1;
    return 2;
}

int stmt_early_int_two(int a) {
    if (a) return 1;
    if (a) return 2;
    return 3;
}

void stmt_early_void(int a) {
    if (a) return;
    g();
}
