void h();
struct S { void (*f)(); void (*g)(); };
S s = { 0, h };
