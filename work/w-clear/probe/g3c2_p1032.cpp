// w-clear grid cell g3c2_p1032 — 3 guard(s), perm [1, 0, 3, 2], 2 call(s)
void g(void *, void *, void *, void *);
void h();
int f(void *a0, void *a1, void *a2, void *a3) { if (a0 == 0) return 5; if (a1 == 0) return 11; if (a2 == 0) return 17; g(a1, a0, a3, a2); h(); return 0; }
