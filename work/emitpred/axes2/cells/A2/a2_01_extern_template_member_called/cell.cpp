// A2 — explicit instantiation DECLARATION (extern template); one member ODR-used.
template <class T> struct H { T a(T x) { return x*3+1; } T b(T x) { return x+7; } };
extern template struct H<int>;
extern int sink(int);
int anchor(int x) { H<int> h; return sink(h.a(x)); }
