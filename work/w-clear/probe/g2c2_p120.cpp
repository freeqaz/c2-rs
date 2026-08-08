// w-clear grid cell g2c2_p120 — 2 guard(s), perm [1, 2, 0], 2 call(s)
void g(void *, void *, void *);
void h();
int f(void *a0, void *a1, void *a2) { if (a0 == 0) return 5; if (a1 == 0) return 11; g(a1, a2, a0); h(); return 0; }
