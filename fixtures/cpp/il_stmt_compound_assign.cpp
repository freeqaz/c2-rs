// IL statement-grammar characterization: COMPOUND assignment.
//
// `x += k` against its hand-expanded equivalent `x = x + k` in the same TU. If
// the front end desugars, the two bodies are byte-identical apart from the
// symbol tokens; if there is a dedicated read-modify-write opcode, they are not.
// `*=` and `<<=` check that the answer is not specific to `+`.
//
// See docs/IL_STMT_GRAMMAR.md §5.

int stmt_cadd(int a) {
    int x = a;
    x += 3;
    return x;
}

int stmt_cadd_expanded(int a) {
    int x = a;
    x = x + 3;
    return x;
}

int stmt_cmul(int a) {
    int x = a;
    x *= 3;
    return x;
}

int stmt_cshl(int a) {
    int x = a;
    x <<= 3;
    return x;
}
