// **Negative** — the two TU-level gates a widened `.gl` symbol index walks right
// past, kept as a file because both are cases where the *census* says the
// functions are in class and `IlBundle::functions` refuses the whole TU. The
// per-function cross-check (roadmap #44, `ROADMAP.md` §6c) cannot see either of
// them, so nothing else in the harness records that they exist.
//
//   * `tf23<int>()` — a function-template callee makes c2 splice
//     `/alternatename:??$tf23@H@@YAXXZ=…` into `.drectve`. The directive count goes
//     from `01` to `02`, `drectve_is_boilerplate` refuses, and it must: the port
//     emits `.drectve` as a constant, so a longer one shifts every later section
//     and the obj diverges at file offset 8 (`PointerToSymbolTable`) no matter how
//     right the codegen is. Measured alone: census **1/1 in class**,
//     `Port=NotImplemented`.
//   * `??1BL23@@QAA@XZ` is **defined in this TU** and delegated to. c2 may inline a
//     locally-defined callee — `il_call_local_def` measured it cloning the body and
//     emitting **no relocation at all** — so a resolved callee that is also a
//     defined name refuses wholesale.
//
// Neither refusal is about the binding: both callees resolve. That is the point —
// a name the index binds correctly is still not a name the port may emit against.

template <class T> void tf23();
void n_template_callee() { tf23<int>(); }

struct BL23 { ~BL23(); int x; };
BL23::~BL23() {}
struct DL23 : BL23 { ~DL23(); int y; };
DL23::~DL23() {}
