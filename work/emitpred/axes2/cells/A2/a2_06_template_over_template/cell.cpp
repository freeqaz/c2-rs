// A2 — templates over templates; only one path through the nest is ODR-used.
template <class T> struct In { T get(T x) { return x*3+1; } T unused(T x) { return x-1; } };
template <class T> struct Out { In<T> i; T call(T x) { return i.get(x); } T never(T x) { return i.unused(x); } };
extern int sink(int);
int anchor(int x) { Out<int> o; return sink(o.call(x)); }
