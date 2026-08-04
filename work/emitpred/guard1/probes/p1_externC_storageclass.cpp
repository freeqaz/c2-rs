// guard1 probe P1 (POST-HOC, outside the graded set).
// Storage-class `extern` INSIDE an extern "C" block, then an extern "C" inline
// definition, unreferenced. Separates storage-class extern from language linkage.
extern "C" { extern int cand(int x); }
extern "C" inline int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
