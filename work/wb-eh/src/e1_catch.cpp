// E1 — a catch funclet only.
int g(int);
int f(int a){ try { return g(a); } catch(int e) { return e+1; } }
