// w-clear grid cell g1c1_p1032 — 1 guard(s), perm [1, 0, 3, 2], 1 call(s)
void g(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) { if (a0 == 0) return 5; g(a1, a0, a3, a2); return 0; }
