#include "hh.h"
extern int sink(int); extern C* pc;
int anchor(int x) { return pc->w(x) + sink(x); }
