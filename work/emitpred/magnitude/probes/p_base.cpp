#include "hh.h"
extern int sink(int); extern B* pb;
int anchor(int x) { return pb->bv(x) + sink(x); }
