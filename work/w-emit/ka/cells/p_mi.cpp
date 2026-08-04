#include "hh.h"
extern int sink(int); extern MI* pm;
int anchor(int x) { return pm->v(x) + pm->q(x) + sink(x); }
