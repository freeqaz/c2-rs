// w-clear grid cell g2c1_p021 — 2 guard(s), perm [0, 2, 1], 1 call(s)
void g(void *, void *, void *);
int f(void *a0, void *a1, void *a2) { if (a0 == 0) return 5; if (a1 == 0) return 11; g(a0, a2, a1); return 0; }
