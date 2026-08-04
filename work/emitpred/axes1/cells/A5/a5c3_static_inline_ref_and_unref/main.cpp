static inline int candU(int x) { return x*3+1; }
static inline int candR(int x) { return x*5+2; }
extern int sink(int);
int anchor(int x) { return candR(x) + sink(x); }
