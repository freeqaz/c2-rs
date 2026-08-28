// CONTROL — three ordinary functions, no EH construct of any kind.
// Establishes that sched0 == after0 on this probe's own compilation, so a
// difference in s7_seh.cpp is attributable to the construct and not to the
// harness.
int ctl_a(int x) { return x + 1; }
int ctl_b(int x, int y) { int s = 0; for (int i = 0; i < y; ++i) s += x + i; return s; }
int ctl_c(int x) { if (x > 3) return x * 2; return x - 2; }
