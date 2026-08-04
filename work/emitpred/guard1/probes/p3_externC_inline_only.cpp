// guard1 probe P3 (POST-HOC) — control, restates a5c1: extern "C" inline, no
// storage-class extern anywhere, unreferenced.
extern "C" inline int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
