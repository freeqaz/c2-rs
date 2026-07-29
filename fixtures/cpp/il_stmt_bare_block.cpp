// IL statement-grammar characterization: a BARE block `{ }`.
//
// A braced block with no control flow attached is the cleanest possible probe
// for the scope-open / scope-close markers: it adds a scope and nothing else.
// The ladder is nested / sequential / empty, so the markers can be counted.
//
// See docs/IL_STMT_GRAMMAR.md §6.

extern void g();

void stmt_block_one() {
    { g(); }
}

void stmt_block_nest() {
    { { g(); } }
}

void stmt_block_two() {
    { g(); }
    { g(); }
}

void stmt_block_empty() {
    {}
}

int stmt_block_local(int a) {
    { int x = a; a = x; }
    return a;
}
