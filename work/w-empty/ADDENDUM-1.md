# w-empty — ADDENDUM 1 (2026-08-06)

Written after GRID-1 (stamp `fea9877e`, 32 cells) was compiled and graded, and
**before** any cell of GRID-2 exists. `PREREG.md` above this line is unedited.

## What GRID-1 settled, and why a second grid is needed

GRID-1 answered "does E fire" on 22 cells and "what does the caller look like"
on the F axis. Three of its results change what the shipped predicate has to
say, and each opens a cell GRID-1 does not contain:

1. **The port's `empty-body` decode is NARROWER than c2's E.** c2 applies E to
   `void g(int a){ int x = a; }`, `void g(int a){ if(a){} }`, `void g(){ return; }`
   and an empty `for` loop; the IL parser **refuses all four**, so the port
   cannot establish emptiness there and must keep today's branch. The shipped
   rule is therefore *"the callee's IL body decodes as `empty-body`"*, which is a
   **subset** of c2's rule, and the difference is published rather than guessed
   at.
2. **`f09_fnptr` is the hazard.** `void g(){} void f(){ void(*p)()=g; p(); }` —
   the callee IS `empty-body`, c2 emits `b ?g` **with** a REL24, and E does
   **not** fire. The port is safe today only because the IL parser refuses that
   caller (`expr-call-in-expr-data-addr-then-plain-call-whole`). A future
   widening of that production would put a wrong emit one line away, so it is
   pinned by a committed test rather than by this paragraph.
3. **`c19_ret_param` is the discriminator and it COSTS us a match.** At the
   workload's flags `int g(int a){return a;}` gives a caller of one `blr` word —
   observationally identical to E — and it is mechanism I. The IL says
   `straight-line`, not `empty-body`, so the shipped rule refuses it and the
   function stays `differs`. **That is the intended direction** and it is why
   `differs` may fall by less than family A entire.

## GRID-2 — the cells the shipped rule needs and GRID-1 does not have

Same instrument, same two compilations, same per-cell anchor control.

| cell | question |
|---|---|
| `g01_data_addr_arg` | the caller's argument is the address of a named global — does the address computation survive when the call is dropped? |
| `g02_float_arg` | the argument is in the FP register file |
| `g03_define_after_use` | the callee is **defined below** the caller |
| `g04_addr_also_taken` | the callee is empty, the call is dropped, and the callee's address is stored elsewhere so it must still be emitted |
| `g05_const_arg` | the argument is a literal (`g(5)`), so the setup is an `li` |
| `g06_three_args` | three formals passed straight through |
| `g07_empty_calls_empty` | the callee is empty and is itself called by a third empty function — two E edges in one TU |
| `g08_empty_ext_decl` | the callee is declared `extern` first and defined empty later, so the caller's declaration is a forward one |

## Registered before compiling one of them

* **A1** — every GRID-2 cell whose callee decodes `empty-body` and whose caller
  is a direct call site is graded **E**, and the caller's whole `.text` is a
  single `4e800020`. **LOSS**: any such cell whose caller keeps a longer body
  (`g01` is the most likely — a global's address is not obviously dead) or a
  REL24.
* **A2** — the shipped predicate refuses at any cell A1 loses on. Concretely: if
  `g01` keeps its address computation, the rule carries an explicit
  `data_sym.is_none()` condition **and a test that pins it**; if `g01` collapses
  like the rest, no such condition is added and the cell is the reason.
* **A3 — CONTROL** — `g04`'s callee is still emitted as its own COMDAT even
  though the call to it is dropped. If it is not, "dropped the call" and "dropped
  the function" are being confused and the port's symbol table would be wrong.

No cell below is compiled before this file and `gen_cells2.py` are committed.
