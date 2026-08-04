#include "hh.h"
extern int sink(int); extern C& rc;
int anchor(int x) { return rc.v(x) + sink(x); }
