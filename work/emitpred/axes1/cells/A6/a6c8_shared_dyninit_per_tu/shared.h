#ifndef SHARED_H
#define SHARED_H
extern int seed();
inline int mk(int x) { return x*3+1; }
static int g_v = mk(seed());
#endif
