// IL statement-grammar characterization: `while`.
//
// A loop is the first construct with a BACKWARD branch, so it pins whether the
// branch opcodes carry a direction or a label reference resolved later. The
// bodies differ only in whether the loop body calls out, which separates the
// loop skeleton from its contents.
//
// See docs/IL_STMT_GRAMMAR.md §8.

extern void g();

void stmt_while_call(int a) {
    while (a) {
        g();
        a = a - 1;
    }
}

void stmt_while_bare(int a) {
    while (a) {
        a = a - 1;
    }
}
