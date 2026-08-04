// w-sect / board #174 — MIXED LINKAGE in one `.data`: an external object
// (StorageClass 2, decorated `?p1@@3HA`) and a `static` one (StorageClass 3,
// UNDECORATED `s1`). §6.1's two rows in one section.
// It has to be `.data` and not `.bss`: an uninitialized unreferenced static is
// DROPPED by c2 entirely, so mixed linkage is unreachable in a `.bss` of a
// functionless TU. See wsect_drop_static.cpp.
int p1 = 1;
static int s1 = 2;
