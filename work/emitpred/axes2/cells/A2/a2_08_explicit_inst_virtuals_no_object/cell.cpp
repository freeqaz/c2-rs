// A2 x A9 — explicit instantiation definition of a class template WITH virtuals,
// but no object is ever constructed in the TU.
template <class T> struct V {
  virtual T f(T x) { return x*3+1; }
  virtual T g(T x) { return x+7; }
  virtual ~V() {}
};
template struct V<int>;
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
