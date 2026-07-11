// W4b2-i (honest rejection): `return g(a) + 70000`. The post-op IS ADD, but the
// constant does not fit a single signed-16-bit `addi` immediate (needs an extra
// `addis`), so it is off the verified 0x24-byte frame → parse_framed_call
// refuses; the terminal tail-call gate also refuses → NotImplemented.
int g(int);
int f(int a) { return g(a) + 70000; }
