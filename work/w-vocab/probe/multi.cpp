// w-vocab: every `.gl` record's pinned field against the obj's own CodeView
// type index for the same function. Varied signatures on purpose — distinct
// arglists, a struct parameter, a pointer, a reference, a member function, a
// void return, an overload set, and two functions that SHARE a signature.
struct P { int x; int y; };
struct Q { int a; };
struct M { int m; int mf(int); int mg(P*, int); };
int  a1(int v)                 { return v + 1; }
int  a2(int v)                 { return v + 2; }        // shares a1's signature
int  b1(int u, int v)          { return u + v; }
int  b2(int u, int v, int w)   { return u + v + w; }
int  c1(P* p)                  { return p->x; }
int  c2(Q* q)                  { return q->a; }
int  d1(char* s)               { return (int)*s; }
int  d2(const char* s)         { return (int)*s; }
void e1(int v)                 { (void)v; }
void e2()                      { }
int  f1(P& p)                  { return p.y; }
unsigned g1(unsigned v)        { return v + 1u; }
double h1(double v)            { return v + 1.0; }
int  M::mf(int v)              { return m + v; }
int  M::mg(P* p, int v)        { return m + p->x + v; }
