// W4b2-i (honest rejection): `return g(a) - 1`. Same `.ex` as the accepted
// framed `g(a) + 1` except the post-op opcode is SUB (`0x03`) not ADD (`0x02`)
// — c2 does NOT canonicalize `-1` to `+(-1)`. Subtraction is non-commutative
// and off the verified 0x24-byte ADD frame, so parse_framed_call refuses it;
// the tail-call gate (now terminal-only) also refuses (the call value is
// consumed) → NotImplemented, not a mis-emitted `b g`.
int g(int);
int f(int a) { return g(a) - 1; }
