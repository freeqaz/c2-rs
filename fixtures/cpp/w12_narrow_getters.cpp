// **Positive** (T3) — the indirect-load leaf over a pointee that is NOT 4 bytes.
// Every function here must emit, and the obj must be byte-exact.
//
// The 4-byte class (`il_expr_deref.cpp` / `il_expr_member.cpp`) is one `lwz`. The
// same IL production over a narrower or wider pointee is a *different opcode*, and
// for one case a different *number* of instructions. Captured with the real
// toolchain (`cl.exe /nologo /Ox /GS- /c`, and re-captured with the dc3 workload's
// `/O1 /Oi /EHsc`; identical in both unless noted):
//
//     pointee            T f(T*)                 int f(T*)   (IL `2C 86 41 74 00`)
//     char/signed char   88630000 lbz  r3,0(r3)   89630000 lbz  r11 ; 7d630774 extsb r3,r11
//     unsigned char/bool 88630000 lbz  r3,0(r3)   88630000 lbz  r3,0(r3)     <- free
//     short              a0630000 lhz  r3,0(r3)   (refused — see w12_narrow_neg.cpp)
//     unsigned short     a0630000 lhz  r3,0(r3)   a0630000 lhz  r3,0(r3)     <- free
//     wchar_t            a0630000 lhz  r3,0(r3)   a0630000 lhz  r3,0(r3)
//     long long / u.l.l. e8630000 ld   r3,0(r3)   (not captured — refused)
//
// Three things in that table are the whole point of this fixture, and each has a
// neighbouring case here that would look identical under a plausible wrong rule:
//
// 1. **`lbz`/`lhz` — never `lha`, and never a mask.** A signed 1-byte pointee
//    returned *as* `char` gets no sign extension at all (`g_c_c`), and a signed
//    2-byte pointee returned as `short` gets `lhz`, not `lha` (`g_s_s`). The
//    obvious rule "a signed load sign-extends" is wrong in both. What *does* pay an
//    instruction is the conversion, and only when the IL says so with a `2C`.
// 2. **The r11-then-r3 rule.** When the widening costs an instruction, the load
//    targets the **scratch** register and the `extsb` produces r3
//    (`lbz r11,0(r3) ; extsb r3,r11`). `g_i_c` against `g_c_c` separates that from
//    "load into r3 and extend in place", which would be one plausible byte and one
//    wrong obj. `g_i_c2` puts the pointer in r4 to show the *base* register is
//    unaffected — the destination is r11 whatever the source is.
// 3. **An unsigned narrow widening is free.** `g_i_uc`, `g_i_b`, `g_i_us` and
//    `g_i_w` carry exactly the same `2C 86 41 74 00` as `g_i_c` and emit *nothing*
//    for it, because `lbz`/`lhz` already zero-extend. A rule that emitted an
//    extension per `2C` would add a wrong instruction to four of these; a rule that
//    emitted none would drop a needed one from `g_i_c`. Only the pointee's
//    signedness discriminates, so both sides are pinned here.
//
// `ld` is **DS-form**: the low two bits of its 16-bit field belong to the form, so
// only offsets that are multiples of 4 are representable. `m_q` (offset 16) and
// `t_q` (offset 8) are the witnesses; no struct member can produce an unaligned
// one, which is why the parser refuses rather than rounds (w12_narrow_neg.cpp).
//
// The member forms exist because the offset must fold into the *load's*
// displacement at the load's own width: `s->h` is `a0630006`, not `a0630000` plus
// an add, and the `27` byte-offset-add type carries the **pointee's** width
// (`27 82 43 f0 08` for a `char` member, `27 a8 43 a0 20` for a `const long long`
// one) — a second, independent statement of the width the `30` load announces.
// The parser requires the two to agree; `m_*`/`t_*` are what make that reachable.
//
// `t_*` are `const` member getters, the shape most of a game engine's function
// count actually is. `const` propagates into the load type, so the body carries a
// `2C` that is a cv-strip rather than a widening (`30 a2 11 93 20`
// `2c 82 11 70 00`) — same width, same signedness, no instruction. `n_*` are the
// non-`const` siblings: their `27` type is `a2 43 f0 08` (const-tagged!) over a
// `30 82 11 70` load, so the two cv bits do *not* track each other and neither one
// may be required to.
//
// `u_*` are the `unsigned long long` const getter, which is the one accepted
// (tag, kind) pair — `(A8, 82)` — that the probe TUs never produced. This fixture
// is its only witness, which is the reason it is here rather than only in a doc.
//
// `s_*` are subscripts. The `28` form's literal is a **byte** offset even for a
// narrow element (`p[3]` on `char*` is `88630003`, `p[2]` on `short*` is
// `a0630004`, `p[2]` on `long long*` is `e8630010`) — measured, not assumed; an
// element-index reading would emit the wrong displacement for two of the three.
//
// `a_*` are the conversions the parser's cv-strip test **cannot** tell from an
// identity, because it compares (width, signedness) and these pairs of distinct C++
// types share one accepted (tag, kind): `bool`/`unsigned char` are both `(82, 12)`,
// `char`/`signed char` both `(82, 11)`, `wchar_t`/`unsigned short` both `(84, 22)`.
// Measured at `/O1` — every one is the bare load and nothing else
// (`88630000 4e800020`, `a0630000 4e800020`), so treating them as free is right.
// The direction it would be *wrong* in is `bool` as the conversion's **target**,
// which c2 does not convert but *normalizes*; that neighbour is pinned as a
// refusal in `w12_narrow_neg.cpp` (`nw_bool_from_uchar`) and it is the only thing
// keeping this rule from being a wrong-bytes emit.

struct S {
    int a;
    char c;
    short h;
    unsigned char u;
    long long q;
};

struct C {
    char c;
    unsigned char u;
    bool b;
    short h;
    unsigned short uh;
    wchar_t w;
    long long q;
    unsigned long long uq;

    char t_c() const;
    unsigned char t_u() const;
    bool t_b() const;
    short t_h() const;
    unsigned short t_uh() const;
    wchar_t t_w() const;
    long long t_q() const;
    unsigned long long u_uq() const;
    int t_i_c() const;
    int t_i_u() const;
    int t_i_b() const;
    int t_i_uh() const;
    int t_i_w() const;
    char n_c();
    unsigned char n_u();
    short n_h();
    long long n_q();
};

// ---- 1. the pointer-parameter matrix, T f(T*) -------------------------------
char g_c_c(char* p) { return *p; }
signed char g_sc_sc(signed char* p) { return *p; }
unsigned char g_uc_uc(unsigned char* p) { return *p; }
bool g_b_b(bool* p) { return *p; }
short g_s_s(short* p) { return *p; }
unsigned short g_us_us(unsigned short* p) { return *p; }
wchar_t g_w_w(wchar_t* p) { return *p; }
long long g_ll_ll(long long* p) { return *p; }
unsigned long long g_ull_ull(unsigned long long* p) { return *p; }

// ---- 2. the widening matrix, int f(T*) --------------------------------------
int g_i_c(char* p) { return *p; }            // lbz r11 ; extsb r3,r11
int g_i_sc(signed char* p) { return *p; }    // same — `char` and `signed char`
                                             // share kind 11 and differ only in id
int g_i_c2(int a, char* p) { return *p; }    // base r4, destination still r11
int g_i_uc(unsigned char* p) { return *p; }  // lbz r3 — the 2C is free
int g_i_b(bool* p) { return *p; }            // lbz r3
int g_i_us(unsigned short* p) { return *p; } // lhz r3
int g_i_w(wchar_t* p) { return *p; }         // lhz r3

// ---- 3. member getters through an explicit pointer (offset folding) ---------
char m_c(S* s) { return s->c; }               // 88630004
int m_i_c(S* s) { return s->c; }              // 89630004 ; extsb
short m_h(S* s) { return s->h; }              // a0630006
unsigned char m_u(S* s) { return s->u; }      // 88630008
int m_i_u(S* s) { return s->u; }              // 88630008
long long m_q(S* s) { return s->q; }          // e8630010 — DS-form, 16 % 4 == 0

// ---- 4. member getters through `this` (const and non-const) ----------------
char C::t_c() const { return c; }
unsigned char C::t_u() const { return u; }
bool C::t_b() const { return b; }
short C::t_h() const { return h; }
unsigned short C::t_uh() const { return uh; }
wchar_t C::t_w() const { return w; }
long long C::t_q() const { return q; }
unsigned long long C::u_uq() const { return uq; }
int C::t_i_c() const { return c; }
int C::t_i_u() const { return u; }
int C::t_i_b() const { return b; }
int C::t_i_uh() const { return uh; }
int C::t_i_w() const { return w; }
char C::n_c() { return c; }
unsigned char C::n_u() { return u; }
short C::n_h() { return h; }
long long C::n_q() { return q; }

// ---- 5. subscripts: the `28` form's literal is a BYTE offset ----------------
char s_c(char* p) { return p[3]; }            // 88630003
unsigned char s_uc(unsigned char* p) { return p[3]; }
short s_h(short* p) { return p[2]; }          // a0630004
unsigned short s_uh(unsigned short* p) { return p[2]; }
long long s_q(long long* p) { return p[2]; }  // e8630010
int s_i_c(char* p) { return p[3]; }           // 89630003 ; extsb
int s_i_uc(unsigned char* p) { return p[3]; } // 88630003

// ---- 6. distinct types that share one accepted (tag, kind) pair -------------
unsigned char a_uc_b(bool* p) { return *p; }      // 88630000 — (82,12) both sides
signed char a_sc_c(char* p) { return *p; }        // 88630000 — (82,11) both sides
char a_c_sc(signed char* p) { return *p; }        // 88630000
wchar_t a_w_us(unsigned short* p) { return *p; }  // a0630000 — (84,22) both sides
unsigned short a_us_w(wchar_t* p) { return *p; }  // a0630000
