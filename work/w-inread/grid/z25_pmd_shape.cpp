void copier();
struct P { int m; int p; int v; };
struct C { unsigned a; const void *t; P d; int s; void (*f)(); };
C c = { 0, 0, { 0, -1 }, 268, copier };
