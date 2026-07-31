// WAE **negative** — every destination the assignment class will not fold, and
// the one route that used to skip the question entirely.
//
// The gate: a destination is admitted only on positive evidence that it is
// register-resident — a formal from `.ex`'s `2D` list, or an automatic plain
// unqualified `int` local whose address is never taken, from `.sy`. Everything
// else is a memory object or a value this class cannot prove it may coalesce,
// and folding its store away is a silently dropped write.
//
// Every function here is out of class under `assign-dst-not-formal-0x26`, which
// is the WAE key: raised at the `26` that pushed the destination (the old
// spelling was `assign-dst-not-formal:eof`, a `byte: None` rendering that
// claimed an end-of-segment the parse had not reached).
//
// `wae_neg_call_global` is the one that was NOT refused before. The
// right-hand-side-is-a-call route hands `dst` straight to the call shape as a
// bound token, so this body censused **in class** as an `int-tail-call` and the
// store to `gv` was folded into thin air. It was never a live mis-emit — a TU
// whose `.gl` carries an unclaimed data symbol is refused whole
// (`il_gl_data_symbol.cpp`) — but the function-level class must not be sound
// only by a translation-unit accounting rule about something else.

int gv;
static int sv;
int wae_neg_g(int a);

int wae_neg_global(int a) { gv = a; return a; }
int wae_neg_file_static(int a) { sv = a; return a; }
int wae_neg_unsigned_local(int a) { unsigned x; x = a; return a; }
int wae_neg_volatile_local(int a) { volatile int v; v = a; return a; }
int wae_neg_fn_static(int a) { static int k; k = a; return a; }
int wae_neg_call_global(int a) { gv = wae_neg_g(a); return gv; }
