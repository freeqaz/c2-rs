// smoke cell: does the pipeline compile, and can both COMDATs be read?
struct V { int* a; int* b; };
int* endv(V* v) { return v->b; }
int* backv(V* v) { return endv(v) - 1; }
void ext_anchor();
void anchor() { ext_anchor(); }
