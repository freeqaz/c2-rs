#include "d5.h"
extern int sink(int);
int anchor(int x) { return topcand(x) + sink(x); }
