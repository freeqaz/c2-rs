// w-decouple — BOTH COFF name encodings in one symbol table.
//
// The writer's branch is `if name.len() <= 8 { b.name8(name) } else { intern }`,
// and interning appends to the string table, which sits at the end of the file
// and whose length is a header field. A TU carrying only short names never
// exercises the interaction; this one puts a short DEFINED name, a long DEFINED
// name, a short UNDEFINED external and a long UNDEFINED external in one obj, so
// the string-table offsets have to survive symbols that do not use it.
//
// `src/xdk/nuispeech/mmio.cpp` is the live shape: eleven defined `extern "C"`
// records of which `mmioSeek`, `mmioRead` and `mmioFlush` are eight or fewer
// and the rest are longer, plus `?FreeHandle@@YAXPAX@Z` undefined.

extern "C" {
int shcal(int, int);
int a_long_undefined_callee(int, int);

int shdef(int a, int b) { return a_long_undefined_callee(a, b); }
int a_long_defined_name(int a, int b) { return shcal(a, b); }
}
