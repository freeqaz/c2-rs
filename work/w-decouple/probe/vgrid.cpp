// GRID-V — one TU carrying every linkage/variadic combination whose `.gl`
// defined-record FLAGS byte (`name_nul + 5`) this lane needs read, so the bit
// is measured across the family rather than inferred from one pair.
//
// Short `extern "C"` names are deliberate: they are exactly the class the
// widened binding walk newly admits, and the class `mangled_is_varargs` cannot
// answer about.
extern "C" {
int v_s(int a, ...) { return a + 1; }          // extern "C" VARIADIC, 3-byte name
int n_s(int a) { return a + 1; }               // extern "C" plain,    3-byte name
int v_long_name_here(int a, ...) { return a + 1; }  // extern "C" VARIADIC, long name
int n_long_name_here(int a) { return a + 1; }       // extern "C" plain,    long name
__forceinline int fi_s(int a) { return a + 1; }     // extern "C" __forceinline
static int st_s(int a) { return a + 1; }            // static
}
int cppv(int a, ...) { return a + 1; }         // C++ VARIADIC  (name ends ZZ)
int cppn(int a) { return a + 1; }              // C++ plain
static int use(int a) { return st_s(a) + fi_s(a); }
int keep(int a) { return use(a); }
