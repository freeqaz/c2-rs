#include "top.h"
extern int sink(int);
int anchor(int x) { return cand(x) + sink(x); }
