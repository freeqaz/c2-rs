// w-fence2 GRID-K — the `.gl` DEFINED-record linkage byte, one cell per linkage.
//
// `gl.rs`'s `linkage_needs_a_directive` documents `05` external / `03` internal
// / `09` dllexport from a three-cell probe and says in as many words that "the
// defined-record value set could not be separated" from the callee and vtable
// runs. This separates it: every name below IS a defined record with a framed
// body-start offset, and nothing here is a declaration.
extern "C" {
void k_ext_a(int *p) { *p = 1; }
static void k_stat_a(int *p) { *p = 2; }
__forceinline void k_cfi_a(int *p) { *p = 3; }
static __forceinline void k_sfi_a(int *p) { *p = 4; }
__declspec(dllexport) void k_exp_a(int *p) { *p = 5; }
void k_user(int *p) {
    k_ext_a(p);
    k_stat_a(p);
    k_cfi_a(p);
    k_sfi_a(p);
}
}

int k_cpp_ext(int a) { return a + 1; }
static int k_cpp_stat(int a) { return a + 2; }
inline int k_cpp_inline(int a) { return a + 3; }
__forceinline int k_cpp_fi(int a) { return a + 4; }

struct KS {
    int m_in(int a) { return a + 5; }  // implicitly inline, COMDAT
    int m_out(int a);
};
int KS::m_out(int a) { return a + 6; }

int k_cpp_user(int a) {
    KS s;
    return k_cpp_ext(a) + k_cpp_stat(a) + k_cpp_inline(a) + k_cpp_fi(a) + s.m_in(a) + s.m_out(a);
}
