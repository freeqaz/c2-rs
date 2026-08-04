// A2 — explicit instantiation definition of the OUTER template; its never-referenced
// member is the only thing that reaches the inner template's unused member.
template <class T> struct In { T get(T x) { return x*3+1; } T unused(T x) { return x-1; } };
template <class T> struct Out { In<T> i; T call(T x) { return i.get(x); } T never(T x) { return i.unused(x); } };
template struct Out<int>;
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
