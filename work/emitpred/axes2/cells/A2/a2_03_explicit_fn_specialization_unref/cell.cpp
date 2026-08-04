// A2 — explicit SPECIALIZATION of a function template, never referenced.
template <class T> T cand(T x) { return x*3+1; }
template <> int cand<int>(int x) { return x+7; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
