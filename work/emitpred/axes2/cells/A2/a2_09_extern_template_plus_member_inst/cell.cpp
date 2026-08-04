// A2 — extern template on the class, plus a member-level explicit instantiation
// definition of the member that is never referenced.
template <class T> struct H { T a(T x) { return x*3+1; } T b(T x) { return x+7; } };
extern template struct H<int>;
template int H<int>::b(int);
extern int sink(int);
int anchor(int x) { H<int> h; return sink(h.a(x)); }
