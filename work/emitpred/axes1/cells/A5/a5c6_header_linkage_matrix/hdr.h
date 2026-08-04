#ifndef HDR_H
#define HDR_H
inline int hiR(int x) { return x*3+1; }
inline int hiU(int x) { return x*3+2; }
static inline int hsiR(int x) { return x*5+1; }
static inline int hsiU(int x) { return x*5+2; }
extern "C" inline int hciR(int x) { return x*7+1; }
extern "C" inline int hciU(int x) { return x*7+2; }
#endif
