#include "hdr.h"
int (*g_p)(int) = &hsi;
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
