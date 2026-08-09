// w-wordwrap GRID T — the STORE WIDTH table, one cell per declared type.
// The class carries the store opcode as a measured mapping from the IL TYPE
// triple, and this file is where every row of that mapping comes from.  A type
// that is not a row here is refused by the recognizer.
//
//     work/w-wordwrap/probe.sh probe/gtype.cpp /O1 /Oi /GS- /c

signed char t_sc;
unsigned char t_uc;
short t_s;
unsigned short t_us;
int t_i;
unsigned int t_u;
long t_l;
unsigned long t_ul;
long long t_ll;
unsigned long long t_ull;
bool t_b;
wchar_t t_wc;
float t_f;
double t_d;
void *t_p;
enum E { E0, E1 };
E t_e;

void T_sc(signed char x) { t_sc = x; }
void T_uc(unsigned char x) { t_uc = x; }
void T_s(short x) { t_s = x; }
void T_us(unsigned short x) { t_us = x; }
void T_i(int x) { t_i = x; }
void T_u(unsigned int x) { t_u = x; }
void T_l(long x) { t_l = x; }
void T_ul(unsigned long x) { t_ul = x; }
void T_ll(long long x) { t_ll = x; }
void T_ull(unsigned long long x) { t_ull = x; }
void T_b(bool x) { t_b = x; }
void T_wc(wchar_t x) { t_wc = x; }
void T_f(float x) { t_f = x; }
void T_d(double x) { t_d = x; }
void T_p(void *x) { t_p = x; }
void T_e(E x) { t_e = x; }
