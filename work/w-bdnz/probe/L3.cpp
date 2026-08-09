// op family, operand = formal k
int op_add(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s += k; return s; }
int op_sub(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int op_mul(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int op_and(int n, int k) { int s = -1; for (int i = 0; i < n; ++i) s &= k; return s; }
int op_or (int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s |= k; return s; }
int op_xor(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s ^= k; return s; }
int op_shl(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s <<= k; return s; }
int op_shr(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s >>= k; return s; }
int op_div(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s /= k; return s; }
// operand = literal
int lit_sub(int n) { int s = 0; for (int i = 0; i < n; ++i) s -= 3; return s; }
int lit_mul(int n) { int s = 1; for (int i = 0; i < n; ++i) s *= 3; return s; }
int lit_xor(int n) { int s = 0; for (int i = 0; i < n; ++i) s ^= 3; return s; }
