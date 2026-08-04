extern "C" inline int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return cand(x) + sink(x); }
