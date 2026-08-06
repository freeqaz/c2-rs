// GRID-T cell t13_size_boundary — S7 — a callee big enough to cross the 64-byte bound if the port can lower it at all. If the port refuses it, S7 never binds and that is PRINTED, not claimed as a pass
int g(int a, int b, int c, int d) {
  return a*b + b*c + c*d + d*a + a*c + b*d + a*a + b*b + c*c + d*d
       + a*b*c + b*c*d + c*d*a + d*a*b + a*b*c*d;
}
int f(int a, int b, int c, int d) { return g(a, b, c, d); }

void ext_anchor();
void anchor() { ext_anchor(); }
