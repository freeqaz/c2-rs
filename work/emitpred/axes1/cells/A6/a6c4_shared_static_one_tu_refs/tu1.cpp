#include "shared.h"
extern int sink(int);
int anchor1(int x) { return sa(x) + sink(x); }
