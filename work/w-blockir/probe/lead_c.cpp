int gz(int);
void loopC(unsigned int n, const float *a, const float *b, float *c) { if (n == 0) return; for (unsigned int i = 0; i < n; i++) { c[i] = a[i] * b[i]; } }
int framed(int a) { return gz(a) + 7; }
