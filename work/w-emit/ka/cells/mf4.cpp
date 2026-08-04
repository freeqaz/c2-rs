#include "hh.h"
extern int sink(int); extern C* pc;
typedef int (C::*pmf)(int);
pmf table[1] = { &C::v };
int anchor(int x) { return sink(x); }
