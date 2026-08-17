// x01 — POST-FREEZE DIAGNOSTIC (not a registered grid cell): g12's class
// template with m referenced only from an unemitted inline; no explicit
// instantiation. Separates "record exists because referenced" from
// "record exists because provide-always".
template <class T> struct C { int m() { return 1; } };
inline int u() { C<char> c; return c.m(); }
