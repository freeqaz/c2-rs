#include "shared.h"
extern int sink(int);
extern C* pc;
int anchor2(int x) { return pc->v(x) + sink(x); }
