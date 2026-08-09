extern "C" {
int cbx(int a) { return a + 1; }
int cfx(int a) { return cbx(a + 1); }
}
