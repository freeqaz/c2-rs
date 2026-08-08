// w-value — the member-call VALUE model, POSITIVE cells.
//
// Each body puts a member call in genuine EXPRESSION position (something
// precedes it, so the statement dispatch does not take the leading `26` as an
// assignment destination) and then consumes its result with a construct the
// expression layer owns. `parse_expr` walks THROUGH the `26 … BD … 4C`
// production and reports **what is behind the call** instead of the call.
//
// **These are census cells, not codegen cells, and the file says so rather than
// implying it.** Nothing here goes in class and the value model ships zero
// conversions BY CONSTRUCTION: `mcall::eat_call_value` pushes no `IlOp`, `IlOp`
// has no call variant, and a poison at the head of `parse_expr`'s end-of-walk
// guards re-raises the same block the `0x26` arm used to return. What the cells
// fix is *which construct a blocked function is filed under*, which is the whole
// of what board #1534 asked for and never had.
//
// Verified per cell against the parent commit's binary; base and tip keys are in
// `docs/rungs/2026-08-08-w-value.md` §5, and the same table is asserted by
// `expr::tests::the_value_model_moves_the_head_only_past_an_expression_construct`.

struct Obj {
    int Get();
    void Set(int);
};

// P1 — a RELATIONAL behind the call. `expr-call-in-expr-recv-load-then-cmp-eq-whole`
// -> `expr-cmp-eq`, the family `WB_READER_FINDINGS.md` §4.1 prices at "nothing
// for the reader, a value model and a lowering for the emitter".
int wvalue_then_cmp_eq(Obj *p, int a) { return a == p->Get(); }

// P2 — the ORDERED relationals, which are a different `Blocker` and a different
// key. Two cells rather than one because `expr-cmp-lt` and `expr-cmp-eq` are
// separately published rows.
int wvalue_then_cmp_lt(Obj *p, int a) { return a < p->Get(); }
int wvalue_then_cmp_ne(Obj *p, int a) { return a + (p->Get() != 0); }

// P3 — DIVIDE and MODULO behind the call, which carry their operand TYPE into
// the key (`expr-typed-op`, board #816). `…-then-op-0x05` -> `expr-op-0x05-8641`
// and `…-then-op-0x06` -> `expr-op-0x06-8642`: the successor key names the type
// the walk reached the operator with, which the old key could not.
int wvalue_then_div(Obj *p, int a, int b) { return a + p->Get() / b; }
int wvalue_then_mod(Obj *p, unsigned a) { return a % p->Get(); }

// P4 — a `void` call, in class as a `multiarg-tail-call` and byte-exact.
// `CallValue::Void` pushes NOTHING and leaves `cstack_ok` TRUE — the one place
// this model can claim to have followed a token exactly — and this cell is the
// witness that the claim costs no acceptance.
void wvalue_void_call(Obj *p, int a) { p->Set(a); }
