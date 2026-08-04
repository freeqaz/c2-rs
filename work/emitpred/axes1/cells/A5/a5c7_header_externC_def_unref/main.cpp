#include "hdr.h"
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
