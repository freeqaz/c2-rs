// w-clear grid cell g1c1_p210 — 1 guard(s), perm [2, 1, 0], 1 call(s)
void g(void *, void *, void *);
int f(void *a0, void *a1, void *a2) { if (a0 == 0) return 5; g(a2, a1, a0); return 0; }
