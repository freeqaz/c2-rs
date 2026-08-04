// A2 — explicit SPECIALIZATION of one member of a class template, never referenced.
template <class T> struct H { T a(T x) { return x*3+1; } T b(T x) { return x+7; } };
template <> int H<int>::b(int x) { return x-5; }
extern int sink(int);
int anchor(int x) { H<int> h; return sink(h.a(x)); }
