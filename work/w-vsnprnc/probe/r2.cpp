// r2 — the CONTROL: identical but the leaf forwards OUTSIDE the TU.
int gg(int);
int outside(int);
int framed(int a) { return gg(a) + 1; }
int leafi(int a) { return outside(a); }
