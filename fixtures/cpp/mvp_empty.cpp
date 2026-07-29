// A translation unit that defines no functions.
//
// The real dc3 workload contains several of these (license-text TUs, and
// platform files whose entire body is #ifdef'd out for the 360 target): the
// front end still emits a full five-file IL bundle, and c2 still emits a real
// COFF obj — just one with no function bodies in it. That makes this the
// smallest possible *whole-TU* byte-exact target, and the first one the port
// can plausibly reach, since it needs no instruction selection at all.
//
// Deliberately include-free (see fixtures/README.md) and free of any construct
// that emits code: no functions, no initialized globals, no statics.

typedef int c2rs_empty_tu_marker;
