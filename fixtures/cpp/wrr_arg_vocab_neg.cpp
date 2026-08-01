// W-RERANK — the argument-operand vocabulary's REFUSALS.
//
// The positive half (`wrr_arg_vocab.cpp`) shows the emitter admits a pointer, a
// cv-qualified pointer and a class-preserving `2C` at a call argument. On its
// own that cannot tell a correct vocabulary from one that admits everything,
// and "admits everything" is the dangerous direction: #139's repair made the
// completeness measure track this vocabulary, so a measure that followed an
// emitter admitting too much would report `-whole` for bodies the shipping path
// refuses — phantom completeness, invisible to `census/gate disagreement`
// because nothing refuses and nothing mis-emits (`docs/ROADMAP.md` §9.13 E4).
//
// Every function here must census OUT of class and the TU must be
// `Port=NotImplemented`.

struct S {
    int take_i(int a);
    int take_p(int* p);
    int take_d(double d);
    int take_ll(long long v);
};

int free_i(int a);

// --- pointer ARITHMETIC in an argument -------------------------------------
// A pointer may be MOVED and not COMPUTED on: `p + 1` on an `int*` is
// `addi r3,r3,4`, and a modeled chain that added 1 would be wrong bytes rather
// than a gap. `parse_expr`'s guard refuses the whole function, and #139 gave
// the measure the same rule — it had none, and was WIDER than its emitter here.
int arith(S* s, int* p) { return s->take_p(p + 1); }

// --- a real-typed argument --------------------------------------------------
// `double` is outside the operand vocabulary at this position in both readers.
int real_arg(S* s, double d) { return s->take_d(d); }

// --- an eight-byte integer argument ----------------------------------------
int wide_arg(S* s, long long v) { return s->take_ll(v); }

// --- a CROSS-CLASS reinterpret, which is not a class-preserving convert ------
// `int*` -> `int` is a ptr4 -> int4 conversion. It happens to be a bare `blr` on
// this target, but it has never been byte-graded across the widths and argument
// positions this parser reaches, so `eat_value_type` refuses it — and the
// measure must refuse it in exactly the same place, or it counts a body
// complete that the emitter will not take.
int reinterp(S* s, int* p) { return s->take_i((int)p); }
int reinterp_free(int* p) { return free_i((int)p); }
