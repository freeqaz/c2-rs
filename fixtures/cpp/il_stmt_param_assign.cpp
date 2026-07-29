// IL statement-grammar characterization: assignment to a PARAMETER.
//
// The destination of an assignment is pushed by some opcode; a parameter and a
// local are different symbol classes, so this pins whether the destination push
// depends on the class. Each pair below writes the same value to a parameter and
// to a local, so the two bodies differ in exactly one symbol.
//
// See docs/IL_STMT_GRAMMAR.md §4.

int stmt_param_assign(int a) {
    a = a + 1;
    return a;
}

int stmt_local_assign(int a) {
    int x = 0;
    x = a + 1;
    return x;
}

int stmt_param_assign_lit(int a) {
    a = 7;
    return a;
}
