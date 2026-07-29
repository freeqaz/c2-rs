// The scalar TYPE table. One identity function per type, so the `.ex` TYPE bytes
// for each can be read straight off the capture with nothing else varying.
// Transcribed in `docs/IL_TYPE_TAGS.md` §2:
//
//   char           82 11 70      int             86 41 74
//   signed char    82 11 10      long            86 41 12
//   unsigned char  82 12 20      unsigned        86 42 75
//   bool           82 12 30      unsigned long   86 42 22
//   short          84 21 11      int*            86 43 f4 08
//   unsigned short 84 22 21      void*           86 43 83 08
//   wchar_t        84 22 71      const char*     86 43 81 20
//   float          86 45 40      double          88 85 41
//
// A TYPE is `<tag> <kind> <LEB128 id>`, so 3 to 5 bytes; the tag encodes width as
// `0x80 + 2*(log2(size)+1)`, the kind the class, and the id picks a member of that
// class out of the TU's type table. That last part is why the three pointer rows
// share `86 43` and differ afterwards — and why census bucket names truncated to
// three bytes silently group distinct pointer types together.
//
// Two census buckets decode directly out of this table:
// `expr-load-type-864383` is `void*` (2.0% of blocked functions) and
// `expr-lit-type-821230` is a `bool` literal (1.0%).
//
// The codegen finding is that identity is a bare `blr` for EVERY row, pointers and
// narrow types included: the ABI hands an argument over already extended, and
// returning it costs nothing. Width only costs where arithmetic happens, which is
// what `il_type_narrow.cpp` measures. So this whole file being out of class is a
// decode limit, not a codegen one.
//
// `t_uint` keeps its explicit `(int)` cast: without it the return type would not
// match the parameter and the fixture would stop being a pure identity. The cast
// puts two `2C` converts in the IL, which is why this one function blocks
// differently from its neighbours.

int t_int(int a) { return a; }
unsigned t_uint(unsigned a) { return (int)a; }
short t_short(short a) { return a; }
unsigned short t_ushort(unsigned short a) { return a; }
char t_char(char a) { return a; }
signed char t_schar(signed char a) { return a; }
unsigned char t_uchar(unsigned char a) { return a; }
bool t_bool(bool a) { return a; }
long t_long(long a) { return a; }
unsigned long t_ulong(unsigned long a) { return a; }
wchar_t t_wchar(wchar_t a) { return a; }
int* t_pint(int* a) { return a; }
void* t_pvoid(void* a) { return a; }
const char* t_pcchar(const char* a) { return a; }
float t_float(float a) { return a; }
double t_double(double a) { return a; }
