extern int cand(int x);
inline int cand(int x) { return x*3+1; }
extern inline int cand2(int x) { return x*7+4; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
