#include "hdr.h"
extern int sink(int);
int anchor(int x) { return hiR(x) + hsiR(x) + hciR(x) + sink(x); }
