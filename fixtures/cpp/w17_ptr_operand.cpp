// **Positive** — the 4-byte pointer OPERAND. Every function here must emit, and
// the whole obj must be byte-exact.
//
// `docs/IL_CALL_IN_EXPR.md` §21. `parse_expr`'s LOAD and LIT positions took an
// int-like TYPE only (`86 41 74` and its three siblings), so every body whose
// value passed through a pointer refused on the type before anything else could
// be judged. That was the largest thing in the census by a factor of 4.7 —
// `expr-load-type-A643` 666,907 functions plus `expr-load-type-8643` 316,800,
// 45.9 % of the blocked workload — and it is a *gate in front of* the pointer
// expression layer, not the layer itself: opening it releases 1,015,144 functions
// from the type keys, of which 14,016 become whole-body complete and the rest
// stop one token deeper (`27` sub-object address, `2C` convert, `99` by-value
// bind).
//
// ## Why this is decode-only
//
// A 4-byte pointer in a register is a 4-byte int in a register. Nothing below
// emits an instruction the port did not already emit for the int spelling of the
// same body: the argument goes in its argument register, the result is already in
// r3, and the null pointer constant is the `li r3,0` an int 0 already produced.
// The TYPE in these positions is an annotation on a *value*, not a selector for a
// load width — that is `shapes.rs`'s `30`, which is gated by `is_ptr_to_4` /
// `value_class` and is untouched by this rung.
//
// ## Why the widening is four positions and not one
//
// A real call site spells the pointer twice: at the `B9` LOAD and again at the
// `55 <TYPE>` that carries the formal's declared type
// (`… B9 p 86 43 f4 08 · 55 86 43 f4 08 · 4C`, captured). Widening only
// `parse_expr` is therefore worth exactly nothing — MEASURED: it moved 1,013,468
// functions between census keys and moved the numerator by **0**. The `41`
// result type is the third, for a body that returns the pointer; `t_zero` and
// `t_id_*` are the ones that would refuse without it.
//
// ## What each function discriminates
//
// `t_p` … `t_v8` — the POINTEE, across every width the target has (1, 2, 4, 8)
//   and across `void`, a pointer-to-pointer and a code pointer. The pointee width
//   is what pointer arithmetic scales by, and it is what the tag carries in the
//   `27` position — so it is the field most likely to leak into a lowering. Here
//   it must not reach the instruction at all: every one of these is the same
//   `b <callee>`.
//
// `t_pc` / `t_pv` / `t_pcp` — the cv-spellings. `A6` is a const-qualified
//   POINTER (`int* const`, and `this`), MEASURED here by `t_pcp`; a const-qualified
//   POINTEE (`const int*`, `t_pc`) keeps the plain `86` tag and differs only in the
//   type-table id. That is why the census head was two rows and not one, and
//   `t_pcp` is the only in-class witness for the larger of the two.
//
// `t_null` / `t_zero` / `t_zeroc` — the LIT position: a null pointer constant,
//   in an argument and as a whole body. `33 86 43 f4 08 00` — a pointer-typed
//   literal whose payload varint is read exactly as an int's.
//
// `t_pa` / `t_ap` / `t_pq` / `t_ppa` — the argument SLOT. "Already in the right
//   register" is the whole reason these are free, so which register it is has to
//   be crossed against pointer-ness: a gate written for the first argument passes
//   `t_p` and every all-pointer case and fails only here.
//
// `t_id_*` — the `41` result position with no call at all: the value is returned
//   straight out of its argument register.
//
// `t_ref` — a REFERENCE parameter passed on by reference. C++ spells no pointer
//   anywhere in it and the IL is a pointer operand throughout, which is exactly
//   the sort of population a source-level reading of this rung would miss.

int  g1(int*);
int  g1c(const int*);
int  g1v(volatile int*);
int  g1cp(int* const);
int  g1ch(char*);
int  g1sh(short*);
int  g1ll(long long*);
int  g1d(double*);
int  g1vd(void*);
int  g1pp(int**);
int  g1f(int (*)(int));
int  g2(int*, int);
int  g2r(int, int*);
int  g3(int*, int*);
int  g4(int*, int*, int);
int  gref(int&);

// ---- the pointee, across every width the target has -------------------------
int t_p    (int* p)              { return g1(p); }
int t_ch   (char* p)             { return g1ch(p); }
int t_sh   (short* p)            { return g1sh(p); }
int t_ll   (long long* p)        { return g1ll(p); }
int t_d    (double* p)           { return g1d(p); }
int t_v8   (void* p)             { return g1vd(p); }
int t_pp   (int** p)             { return g1pp(p); }
int t_fp   (int (*f)(int))       { return g1f(f); }

// ---- the cv-spellings: `86` plain, `86` const POINTEE, `96`/`A6` on the pointer
int t_pc   (const int* p)        { return g1c(p); }
int t_pv   (volatile int* p)     { return g1v(p); }
int t_pcp  (int* const p)        { return g1cp(p); }

// ---- the LIT position -------------------------------------------------------
int t_null ()                    { return g1(0); }
int*   t_zero  ()                { return 0; }
char*  t_zeroc ()                { return 0; }
void*  t_zerov ()                { return 0; }

// ---- the argument slot ------------------------------------------------------
int t_pa   (int* p, int a)       { return g2(p, a); }
int t_ap   (int a, int* p)       { return g2r(a, p); }
int t_pq   (int* p, int* q)      { return g3(p, q); }
int t_ppa  (int* p, int* q, int a) { return g4(p, q, a); }

// ---- a REFERENCE parameter, which is a pointer operand in the IL ------------
int t_ref  (int& r)              { return gref(r); }

// ---- the `41` result position, with no call ---------------------------------
int*   t_id_p  (int* p)          { return p; }
char*  t_id_c  (char* p)         { return p; }
void*  t_id_v  (void* p)         { return p; }
