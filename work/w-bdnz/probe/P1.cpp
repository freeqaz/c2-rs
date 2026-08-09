int p_sub(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int p_mul(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int p_and(int n, int k) { int s = -1; for (int i = 0; i < n; ++i) s &= k; return s; }
int p_or (int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s |= k; return s; }
int p_xor(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s ^= k; return s; }
int p_shl(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s <<= k; return s; }
int p_sar(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s >>= k; return s; }
int p_uns(unsigned n, int k) { int s = 0; for (unsigned i = 0; i < n; ++i) s -= k; return s; }
int p_braced(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) { s -= k; } return s; }
int p_hi(int n, int k) { int s = 32767; for (int i = 0; i < n; ++i) s *= k; return s; }
int p_lo(int n, int k) { int s = -32768; for (int i = 0; i < n; ++i) s *= k; return s; }
