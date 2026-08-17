// g07 — CONTROL for the data-side closure: a namespace-scope function pointer
// whose initializer references an inline function. c2 must emit the function;
// the predicate's no-references clause must read FALSE.
inline int fi(int x) { return x + 2; }
int (*gp)(int) = &fi;
