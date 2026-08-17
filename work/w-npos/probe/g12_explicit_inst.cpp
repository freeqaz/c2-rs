// g12 — THE DANGER CELL for the zero-roots rule: an explicit class template
// instantiation. If c2 emits `C<char>::m` here with zero ordinary roots and
// zero data references, the zero-roots predicate is unsound unless the IL
// distinguishes this TU. Registered two-sided in PREREG P7.
template <class T> struct C { int m() { return 1; } };
template struct C<char>;
