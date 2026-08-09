int gz(int);
void loopB(unsigned int n, float *a, float s) { if (n == 0) return; for (unsigned int i = 0; i < n; i++) { a[i] *= s; } }
int framed(int a) { return gz(a) + 7; }
