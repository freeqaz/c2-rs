// IL statement-grammar characterization: LOCAL DECLARATION + assignment.
//
// Three bodies that differ only in where the initialization happens:
//   decl_split  — declaration and assignment are separate statements
//   decl_init   — declaration with an initializer
//   decl_two    — two initialized declarations, so a second local's token is
//                 visible next to the first
// Together they separate "a declaration emits its own IL" from "a declaration
// is free and only the initializer is a statement".
//
// See docs/IL_STMT_GRAMMAR.md §3.

int stmt_decl_split(int a) {
    int x;
    x = a;
    return x;
}

int stmt_decl_init(int a) {
    int x = a;
    return x;
}

int stmt_decl_two(int a) {
    int x = a;
    int y = x;
    return y;
}
