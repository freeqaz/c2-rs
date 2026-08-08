// w-clear grid cell g1c2_p201 — 1 guard(s), perm [2, 0, 1], 2 call(s)
void g(void *, void *, void *);
void h();
int f(void *a0, void *a1, void *a2) { if (a0 == 0) return 5; g(a2, a0, a1); h(); return 0; }
