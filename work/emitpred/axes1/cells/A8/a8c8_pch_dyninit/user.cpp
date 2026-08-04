#include "pchd.h"
int g_v = mk(seed());
extern int sink(int);
int anchoru(int x) { return sink(x) + g_v; }
