#include "shared.h"
extern int sink(int);
int anchor2(int x) { return cb(x) + sink(x); }
