// W41 — the **framed member call**: `return p->m() - k;`, and the pointer
// conversion the receiver is allowed to carry.
//
// The brief's target was `expr-call-in-expr-recv-load-whole` — 10,494 functions,
// 10,463 of them `calls-1` — scheduled as "the member call preceded by assignment
// statements". **It contains zero of those.** A member-call *production*
// first-blocker histogram over the 878-TU dc3 workload decomposes the row 1:1:
//
//   6,463  return p->m() + n*k;   a value live across the call -> Class B
//   3,559  return p->m() - k;     THIS FILE
//     440  p->m() with a `2C` pointer conversion on the receiver   THIS FILE
//      32  residue
//
// Every one of the 3,559 is a **subtraction**, and the free-function twin
// `return g(a) - k;` is **0** functions on the same workload — which is exactly
// why the `03` byte had never been asked for at `eat_call_postop`, the one
// locator that decodes a call's post-op region. `- k` and `+ k` are the *same
// instruction* with a different immediate; the comment that refused `03` grouped
// it with `04` (MUL, which really does strength-reduce) and the grouping was not
// a measurement.
//
// The emission is `BodyShape::FramedCall`, which needs **no codegen at all**:
// `this` is argument zero exactly as it is for the tail form, so the argument
// setup is the same `select_text` register move and the rest is the shipped
// 0x24-byte frame. Every word below was read off the reference obj before any of
// this was written (`work/w41/probe/p1.cpp`, `p5.cpp` at `/O1 /GS- /c`):
//
//   int f(A* p)             { return p->gi() - 20; }   bl ; 3863ffec addi r3,r3,-20
//   int f(A* p)             { return p->gi() + 20; }   bl ; 38630014 addi r3,r3,20
//   int f(int k, A* p)      { return p->gi() - 20; }   7c832378 mr r3,r4 ; bl ; addi
//   int f(int j,int k,A* p) { return p->gi() - 20; }   7ca32b78 mr r3,r5 ; bl ; addi
//   E*  f(A* p)             { return p->ge() - 1;  }   bl ; addi r3,r3,-20 (sizeof E)
//   int f(A* p)             { return p->gi() + 0;  }   48000000 b ?gi  -- the FOLD
//   int f(A* p)             { return p->gi() - 40000; } addis+addi   -- REFUSED
//   void f(void* v)         { ((S*)v)->m(); }          48000000 b ?m   -- the 2C
//
// Every function here is in class and byte-exact; the neighbours that are not are
// in `w41_framed_member_call_neg.cpp`, one per refusal row.

struct E { int x; int y; int z; int w; int v; };   // 20 bytes, so `- 1` is `- 20`

struct A {
    int  gi();
    int  gic() const;
    E*   ge();
    unsigned ug();
    int  ga(int);
};

struct S { int a; void v(); int g(); };

// --- the row itself: a member call whose result has a literal subtracted -----
int  fm_sub_k    (A* p)                 { return p->gi() - 20; }
int  fm_sub_1    (A* p)                 { return p->gi() - 1; }
int  fm_sub_max  (A* p)                 { return p->gi() - 32767; }
int  fm_sub_min  (A* p)                 { return p->gi() + 32767; }

// --- the ADD half, which the free-function form already emitted -------------
int  fm_add_k    (A* p)                 { return p->gi() + 20; }
int  fm_add_1    (A* p)                 { return p->gi() + 1; }

// --- the receiver at every formal position: the `mr r3,rN` argument setup ---
int  fm_recv_1   (int k, A* p)          { return p->gi() - 20; }
int  fm_recv_2   (int j, int k, A* p)   { return p->gi() - 20; }
int  fm_recv_7   (int a, int b, int c, int d, int e, int f, A* p)
                                        { return p->gi() - 20; }

// --- the result's own type: a pointer, and the pointer scale ----------------
E*   fm_ptr_sub  (A* p)                 { return p->ge() - 1; }
E*   fm_ptr_add  (A* p)                 { return p->ge() + 2; }

// --- cv-qualification on the pointer, the pointee and the method ------------
int  fm_const_m  (const A* p)           { return p->gic() - 20; }
int  fm_const_p  (A* const p)           { return p->gi() - 20; }

// --- an unsigned result -----------------------------------------------------
unsigned fm_unsigned(A* p)              { return p->ug() - 20u; }

// --- the identity FOLD: `+ 0` is not a framed call, it is the tail branch ----
int  fm_fold_add (A* p)                 { return p->gi() + 0; }
int  fm_fold_sub (A* p)                 { return p->gi() - 0; }
int  fm_plain    (A* p)                 { return p->gi(); }

// --- the receiver's pointer conversion, which emits nothing -----------------
void fm_cast_void (void* v)             { ((S*)v)->v(); }
int  fm_cast_ret  (void* v)             { return ((S*)v)->g(); }
int  fm_cast_sub  (void* v)             { return ((S*)v)->g() - 20; }
void fm_cast_const(const S* p)          { const_cast<S*>(p)->v(); }
int  fm_cast_recv1(int k, void* v)      { return ((S*)v)->g() - 20; }

// --- a member function whose own body is one of these -----------------------
struct Holder {
    A* m_a;
    int viaself();
    int viaarg(A* q);
};
int Holder::viaarg(A* q)                { return q->gi() - 20; }

// --- the free-function twin, through the SAME shared post-op locator --------
int  gfree(int);
int  fm_free_sub (int a)                { return gfree(a) - 20; }
int  fm_free_add (int a)                { return gfree(a) + 20; }
