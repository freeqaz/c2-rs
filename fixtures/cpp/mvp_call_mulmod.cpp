// W4b2-i (honest rejection): `return g(a) * 5`. Post-op is MUL (`0x04`), which
// strength-reduces to a different (0x28-byte) body off the verified frame.
// Refused by both parse_framed_call (non-ADD post-op) and the terminal
// tail-call gate (consumed call value) → NotImplemented.
int g(int);
int f(int a) { return g(a) * 5; }
