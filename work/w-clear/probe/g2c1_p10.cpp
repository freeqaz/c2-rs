// w-clear grid cell g2c1_p10 — 2 guard(s), perm [1, 0], 1 call(s)
void g(void *, void *);
int f(void *a0, void *a1) { if (a0 == 0) return 5; if (a1 == 0) return 11; g(a1, a0); return 0; }
