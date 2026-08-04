#include "shared.h"
extern int sink(int);
int anchor1(int x) { C c; return c.v(x) + sink(x); }
