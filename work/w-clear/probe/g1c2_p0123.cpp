// w-clear grid cell g1c2_p0123 — 1 guard(s), perm [0, 1, 2, 3], 2 call(s)
void g(void *, void *, void *, void *);
void h();
int f(void *a0, void *a1, void *a2, void *a3) { if (a0 == 0) return 5; g(a0, a1, a2, a3); h(); return 0; }
