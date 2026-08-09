int gz(int);
void loopA(unsigned int n, const float *a, float *b) { if (n == 0) return; for (unsigned int i = 0; i < n; i++) { b[i] += a[i]; } }
int framed(int a) { return gz(a) + 7; }
