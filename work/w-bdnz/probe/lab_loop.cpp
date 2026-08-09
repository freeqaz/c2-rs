// TEST for the label-lead measurement: this lane's counted loop in the control's
// `lead` slot, everything else byte-for-byte the control's.
int gz(int);
int lead(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int z9(int a) { return gz(a) + 7; }
