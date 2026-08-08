// w-clear grid cell g1c1_p120 — 1 guard(s), perm [1, 2, 0], 1 call(s)
void g(void *, void *, void *);
int f(void *a0, void *a1, void *a2) { if (a0 == 0) return 5; g(a1, a2, a0); return 0; }
