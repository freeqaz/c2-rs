// W41 negative — one case per refusal row of the framed member call, each with
// its measured cost on the 878-TU dc3 workload recorded in the rung document.
// Every function here must census **0/N in class**: `c2rs census` on this file is
// the check that each gate below actually refuses, and `Port=NotImplemented` is
// the check that nothing slipped past it into codegen.

struct E { int x; int y; int z; int w; int v; };

struct A {
    int    gi();
    E*     ge();
    int    ga(int);
    int    g2(int, int);
    float  gf();
    double gd();
};

// n_arg — an explicit argument beside the receiver. `BodyShape::FramedCall`
// carries ONE operand stream, so it can spell "put this formal in r3" and nothing
// else; c2 really does emit a permutation under a frame
// (`int f(A* p,int a,int b){ return p->ga(b) - 20; }` is `mr r4,r5 ; bl ; addi`),
// so this is a real limit and not a restatement of the tail form's.
int n_arg (A* p, int a)          { return p->ga(a) - 20; }
int n_arg2(A* p, int a, int b)   { return p->g2(a, b) - 20; }

// n_wide — the literal does not fit a single signed-16-bit `addi`. MEASURED:
// `± 40000` comes back as `addis r3,r3,±1` followed by `addi`, a second
// instruction and a longer body.
int n_wide (A* p)                { return p->gi() - 40000; }
int n_wide2(A* p)                { return p->gi() + 40000; }

// n_mul — a MULTIPLY post-op, which is the case `03` was wrongly grouped with:
// it strength-reduces to a shift/add sequence and is genuinely not one `addi`.
int n_mul (A* p)                 { return p->gi() * 20; }

// n_twoop — two post-ops. The region is exactly one literal and one operator.
int n_twoop(A* p)                { return p->gi() - 20 - 1; }

// n_classb — a value LIVE ACROSS THE CALL, which is 6,463 of the 10,494-function
// row and the largest single thing this rung does not take. c2 saves the formal
// in r31 with a `std`/`ld` pair and re-materializes it after the branch:
//   std 31,-16(1) ; mr 31,4 ; bl ?ge ; mulli 11,31,20 ; add 3,3,11 ; ld 31,-16(1)
// That is a frame class this port does not have.
E* n_classb (A* p, int n)        { return p->ge() + n; }
int n_classb2(A* p, int n)       { return p->gi() + n; }

// n_nine — nine formals. Past the eighth a parameter is stack-homed and its setup
// is `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole formals
// list, the same predicate `select_text` raises.
int n_nine(int a, int b, int c, int d, int e, int f, int g, int h, A* p)
                                 { return p->gi() - 20; }

// n_fp — a `float`/`double` result with a post-op. The TU carries `_fltused` and
// the post-op is `fadds`, not `addi`; refused at the result type.
float  n_fp (A* p)               { return p->gf() - 20.0f; }
double n_fp2(A* p)               { return p->gd() - 20.0; }

// n_recv_volatile — a `volatile` receiver. c2 homes the parameter in the frame and
// reloads it, so the body is not a leaf at all; the shared operand-type locator
// refuses it, which is how this position inherits `GAPS.md` §6 instance #13.
struct V { void vm(); int vg(); };
int n_recv_volatile(V* volatile p) { return p->vg() - 20; }

// n_recv_global — the receiver is not one of this function's formals, so the
// framed path's register move has nothing to move.
extern A* g_a;
int n_recv_global(int k)         { return g_a->gi() - 20; }

// n_seq — the call is not the last thing in the body. Still refused by name; the
// Class A statement-call sequence with a member call in it is a further rung.
int n_seq(A* p)                  { int r = p->gi(); return r - 20; }
