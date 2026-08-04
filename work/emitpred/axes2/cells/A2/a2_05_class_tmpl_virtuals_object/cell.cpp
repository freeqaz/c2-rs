// A2 — control for a2_02: same class template with virtuals, no extern template.
template <class T> struct V {
  virtual T f(T x) { return x*3+1; }
  virtual T g(T x) { return x+7; }
  virtual ~V() {}
};
extern int sink(int);
extern void use(void*);
int anchor(int x) { V<int> o; use(&o); return sink(x); }
