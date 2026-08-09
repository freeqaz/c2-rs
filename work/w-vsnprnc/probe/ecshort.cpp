// ecshort — extern "C", names that FIT the 8-byte inline field, identity.
extern "C" {
int cal5(int, int, int, int, int);
int fwd5(int a, int b, int c, int d, int e) { return cal5(a, b, c, d, e); }
}
