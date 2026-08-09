// w-xtea probe P3 — is the `80`+8 escape a property of TAG 0x88, or only of the
// 8-byte INTEGER kinds? A double is tag 0x88 too (`88 85 41`), so if the rule is
// "tag 0x88 => 8-byte escape" a double literal must obey it. If it does not, the
// rule must be narrowed to the integer kinds and this probe is why.
// STRUCTURAL axes: type tag, scalar kind (int vs float), and width control.
typedef unsigned long long ull;
double f_dbl(double a)  { return a + 1.5; }        // tag 88 85 (double)  8 B
float  f_flt(float a)   { return a + 1.5f; }       // 4-byte float control
ull    f_shift(ull a)   { return a >> 33; }        // 8-byte lhs, SMALL literal
ull    f_add(ull a)     { return a + 0x100000000ull; }  // 8-byte, escape needed
bool   f_cmp(ull a)     { return a > 0xFFFFFFFFull; }   // literal in a COMPARE
