// **Positive** — the pointer-valued leaf. Every function here must emit, and the
// whole obj must be byte-exact.
//
// This is rung 1 of `docs/IL_LOAD_TYPES.md` §6: the *type* gate on the
// indirect-load leaf widens from "4-byte integer" to "4-byte integer **or**
// 4-byte pointer", and a new identity leaf accepts a pointer returned unchanged.
// It buys **zero new instructions**, which is the whole reason it is the first
// rung — a pointer getter is the same `lwz` an `int` getter is, and a pointer
// identity is the same bare `blr`:
//
//   int  H::gi()  const { return mi; }   ->  80630010  lwz r3,16(r3) ; blr
//   int* H::gpi() const { return mpi; }  ->  80630000  lwz r3,0(r3)  ; blr
//   S*   id_p(S* s)     { return s; }    ->            blr           (r3 already)
//   C*   C::self()      { return this; } ->            blr
//
// so the emitter (`c2_core::codegen::indirect_load_text`, which consumes no type
// at all, and the straight-line `[Load(first param)]` case) is untouched.
//
// ## What each function discriminates
//
// `gp_i` / `gp_c` / `gp_v` — the three pointee widths that all still make the
// loaded *value* a 4-byte pointer. This is the trap the type gate has to get
// right: `char* p` is `lbz` when you load **through** it and `lwz` when you load
// **it**. The gate is on the loaded type's own width (tag nibble 6 = 4 bytes,
// kind class 3 = data pointer), never on the pointee's:
//
//   char  f(char* p)  { return *p; }    30 82 11 70     ->  lbz  (refused, T2)
//   char* f(H* h)     { return h->mpc; } 30 86 43 f0 08 ->  lwz  (accepted here)
//
// `gp_fn` is kind class **4** (function/code pointer, `86 44 …`) rather than 3.
// Same `lwz`; a separate kind byte purely because c1xx types code pointers
// apart. Admitted alongside class 3 because the load is identical and two gates
// for one instruction are two places to drift.
//
// `gp_cv` is the `2C` strip: a member read through a **const** `this` loads as
// `30 A6 43 <id>` and is then converted to the unqualified pointer,
// `2C 86 43 <id> 00`. MEASURED (`IL_LOAD_TYPES.md` §3): a `2C` from pointer to
// pointer emits **nothing** — `void* f(H* p){ return p; }` is a bare `blr`.
// Address-*adjusting* casts never come through `2C`; they are the intrinsics
// 2113/2114/2115, and `w12_ptr_leaf_neg.cpp` holds one to prove it stays out.
//
// `id_p` / `self_np` / `self_c` / `id_void` are the identity leaf:
// `B9 <tok> <ptr> [2C <ptr> 00] 41 <ptr>` with no `30` at all. `return this;`
// is the same production as `return p;` — `this` is simply the token the
// pre-body `B9 … 99 … 00` group binds, and it occupies r3, which is why the
// const and non-const spellings both fold to a bare `blr`. Both spellings are
// here because `this` in a const member function carries the `A6` tag and in a
// non-const one carries `86`, and a gate written for one tag would silently
// refuse the other — the commoner one.
//
// `gc_p` / `gp_cv` read a member through a `const C*`. The captures show which
// position the `const` actually lands in, and it is not the one the name
// suggests: the **base** is `86 43` (the pointer itself is not const, only its
// pointee), while the **loaded** type is `A6 43` and the `2C` strips it —
//
//   b9 <c> 86 43 86 20  33 86 41 74 00  27 86 43 96 20
//   30 a6 43 95 20      2c 86 43 91 20 00  41 86 43 91 20
//
// so `gc_p` exercises the `A6` tag in the `30` position and `self_c` exercises it
// in the base position. Both spellings are needed; neither implies the other.
//
// `deref_pp` (`int** -> int*`) used to be a *negative* in
// `il_expr_load_neg.cpp`, refused with the note that it "emits the same word but
// stays refused" because the loaded type was a pointer. That is precisely the
// gate this rung opens, so it moves here — same `lwz r3,0(r3)`, now accepted.
//
// `off16` keeps the offset near the top of the 16-bit displacement the shared
// tail bounds; a member past it materializes an index register and refuses
// (`n_offbig` in the negatives).

struct S {
    int a;
};

struct H {
    int* mpi;      // offset 0
    char* mpc;     // 4
    void* mpv;     // 8
    int (*mfn)();  // 12
    int mi;        // 16
};

struct C {
    int* mp;
    C* self_np();
    const C* self_c() const;
};

C* C::self_np() { return this; }
const C* C::self_c() const { return this; }

struct Wide {
    int pad[8000];
    int* tail;
};

int* gp_i(H* h) { return h->mpi; }
char* gp_c(H* h) { return h->mpc; }
void* gp_v(H* h) { return h->mpv; }
int* gp_i2(int a, H* h) { return h->mpi; }

int (*gp_fn(H* h))() { return h->mfn; }

int* gc_p(const C* c) { return c->mp; }
void* gp_cv(const C* c) { return c->mp; }

int* deref_pp(int** pp) { return *pp; }

S* id_p(S* s) { return s; }
void* id_void(void* v) { return v; }

int* off16(Wide* w) { return w->tail; }
