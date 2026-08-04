// guard1 probe P2 (POST-HOC). inline definition FIRST, extern declaration AFTER.
// Tests whether the extern-forces-emission effect is entity-level or order-dependent.
inline int cand(int x) { return x*3+1; }
extern int cand(int x);
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
