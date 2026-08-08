// w-clear grid cell g3c1_p0123 — 3 guard(s), perm [0, 1, 2, 3], 1 call(s)
void g(void *, void *, void *, void *);
int f(void *a0, void *a1, void *a2, void *a3) { if (a0 == 0) return 5; if (a1 == 0) return 11; if (a2 == 0) return 17; g(a0, a1, a2, a3); return 0; }
