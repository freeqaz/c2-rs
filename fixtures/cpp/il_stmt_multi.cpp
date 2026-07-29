// IL statement-grammar characterization: a MULTI-STATEMENT body mixing kinds.
//
// Four statements of three different kinds (initialized declaration, call
// assignment, return) in one straight-line body — the shape most real functions
// have, and the one the current single-statement parser cannot reach.
//
// See docs/IL_STMT_GRAMMAR.md §2, §6.

extern int g(int);

int stmt_multi(int a) {
    int x = a + 1;
    int y = x + 2;
    int z = g(y);
    return z;
}
