#include "shared.h"
extern int sink(int);
int anchor2(int x) { return sink(x) + g_v; }
