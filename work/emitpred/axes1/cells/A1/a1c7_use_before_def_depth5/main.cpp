inline int cand(int x);
extern int sink(int);
int anchor(int x) { return cand(x) + sink(x); }
#include "d5.h"
