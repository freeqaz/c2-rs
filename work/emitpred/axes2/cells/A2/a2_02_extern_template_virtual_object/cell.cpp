// A2 — extern template on a class template WITH virtuals; an object is constructed.
template <class T> struct V {
  virtual T f(T x) { return x*3+1; }
  virtual T g(T x) { return x+7; }
  virtual ~V() {}
};
extern template struct V<int>;
extern int sink(int);
extern void use(void*);
int anchor(int x) { V<int> o; use(&o); return sink(x); }
