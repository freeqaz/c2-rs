extern "C" {
int cb(int a) { return a + 1; }
int cf(int a) { return cb(a + 1); }
}
