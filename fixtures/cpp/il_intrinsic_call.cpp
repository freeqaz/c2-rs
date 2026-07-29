// `.ex` opcode 0x40 — the INTRINSIC CALL token (`33 <int> <id> 40 <ret-type>
// <args> 4C`), which the census mislabels `expr-cast`. Each function isolates
// one intrinsic so its id and its c2 expansion can be read off directly.
// Evidence base for docs/IL_CAST_CONVERT.md §1.
typedef unsigned int c2rs_size_t;

extern "C" {
void *memcpy(void *, const void *, c2rs_size_t);
void *memset(void *, int, c2rs_size_t);
int memcmp(const void *, const void *, c2rs_size_t);
c2rs_size_t strlen(const char *);
int strcmp(const char *, const char *);
int abs(int);
double fabs(double);
double sqrt(double);
unsigned int _rotl(unsigned int, int);
}

void *t_memcpy(void *d, const void *s, c2rs_size_t n) { return memcpy(d, s, n); }

void *t_memset(void *d, int c, c2rs_size_t n) { return memset(d, c, n); }

int t_memcmp(const void *a, const void *b, c2rs_size_t n) { return memcmp(a, b, n); }

c2rs_size_t t_strlen(const char *s) { return strlen(s); }

int t_strcmp(const char *a, const char *b) { return strcmp(a, b); }

int t_abs(int a) { return abs(a); }

double t_fabs(double a) { return fabs(a); }

double t_sqrt(double a) { return sqrt(a); }

unsigned int t_rotl(unsigned int a, int n) { return _rotl(a, n); }

// The class-layout family (ids 0x841..0x847): the same 0x40 token, but the
// emitted code depends on the literal offset arguments -- offset 0 is nothing,
// a non-zero base offset is a null-guarded `addi`.
struct A1 {
    int a;
};
struct A2 {
    int b;
};
struct M : A1, A2 {
    int d;
};

A1 *up_zero(M *m) { return m; }

A2 *up_nonzero(M *m) { return m; }

int fld_base(M *m) { return m->b; }
