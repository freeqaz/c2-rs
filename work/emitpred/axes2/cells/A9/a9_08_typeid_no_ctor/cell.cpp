// A9 — typeid on a polymorphic type with no constructor kept.
// `type_info` is declared by hand: this toolchain is invoked with no INCLUDE path.
class type_info { public: const char* name() const; private: type_info(const type_info&); type_info& operator=(const type_info&); };
struct D { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~D() {} };
extern int sink(int);
int anchor(int x) { return typeid(D).name() ? sink(x) : 0; }
