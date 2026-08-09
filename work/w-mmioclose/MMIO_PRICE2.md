# W-MMIOCLOSE — `src/xdk/nuispeech/mmio.cpp`, re-priced against the GATE

`w-ifn`'s `MMIO_PRICE.md` priced this TU at **six** mechanisms, all of them
`mmioClose`'s, and closed with *"`mmioClose`'s 124 bytes are the entire
remaining distance"*. The commission repeated it. **It is not true, and the TU's
own scan row says so in one number.**

Everything below is this lane's own scan, its own captured `.gl`, or a cell it
compiled and graded against real `c2.dll` under wibo.

---

## 1. The number the price was missing

`work/w-mmioclose/base2.jsonl`, `src/xdk/nuispeech/mmio.cpp`:

```
class = vocab-gap    fn_total = 11    fn_in_class = 10
detail = ".ex 5153 B, 1 .gl names — c2_il::functions() and dyninit_tu() both None"
```

> **One `.gl` name, eleven `.ex` segments.**

`c2_il::mangled_names` is `looks_mangled`, which is `contains("@@")`. Ten of
this TU's eleven functions are `extern "C"` and undecorated —
`mmioClose`, `mmioFlush`, `mmioGetInfo`, … — and the eleventh name the scan
counts is the *callee* `?FreeHandle@@YAXPAX@Z`. `Bindings::per_record` binds
nothing unless the `.gl` records are 1:1 with the `.ex` segments, so
`IlBundle::functions()` returns `None` on this TU **before any body is looked
at**.

**Even if all six of `w-ifn`'s mechanisms were paid and `mmioClose` were
byte-exact, `mmio.cpp` would not convert.** That is `docs/CEILING.md` §11's
NC-4 — *"the published refusal names a layer that is not the one that
fails"* — and it is the fifth instance the section has taken since it was
written yesterday.

### 1.1 Why the byte-fraction instrument said otherwise

`frontier-bytefrac-top-accepted` read `256 of 380` at `w-ifn`'s tip, and the
commission read that as *"factors A, B and C hold, so function bytes are the
entire remaining distance"*. The two numbers come from **two different
bindings**:

| | binding | what it decides |
|---|---|---|
| `fnbyte-*` / byte fraction | `EmitBinding::name` (`FnCensus::emit_name`) | per-function grading, one COMDAT at a time |
| **TU `match`** | `Bindings::per_record` via `gl_defined_names_framed` | whether the port emits the OBJ at all |

`w-ifn` moved the first by 192 bytes. The second has never bound this TU. Board
#918 measured those two bindings disagreeing on 74,955 workload rows; this is
that disagreement deciding a lane's target.

---

## 2. The eleven records, and the attribute this lane decoded

`c2rs capture` at the workload's own flags, read by
`c2_il::func::gl::gl_function_attrs`:

| function | attribute | `FN_FLAG_INLINABLE` (0x40) | source |
|---|---:|---|---|
| `mmioGetInfo` | `0x68` | set | — |
| `mmioSetInfo` | `0x68` | set | — |
| `mmioStringToFOURCCW` | `0x68` | set | — |
| **`mmioFlush`** | **`0x28`** | **CLEAR** | `__declspec(noinline)` |
| `mmioSeek` | `0x68` | set | — |
| **`mmioSetBuffer`** | **`0x28`** | **CLEAR** | `__declspec(noinline)` |
| `mmioOpenW` | `0x68` | set | — |
| `mmioClose` | `0x68` | set | — |
| `mmioAdvance` · `mmioRead` · `mmioWrite` | `0x68` | set | — |

**Eleven of eleven, and the two that are clear are exactly the two the dc3
source marks.** The grid in `work/w-mmioclose/probe/glgrid.cpp` is the other
population: nine cells, `inline`, `__forceinline`, `static`,
`static`+`noinline`, and two with real bodies — 6 set, 3 clear, no exception.

---

## 3. `mmioClose`, re-priced — and the six become NINE

`w-ifn`'s six are re-derived against this tree's source, not inherited.

| # | mechanism | this lane's re-derivation |
|---:|---|---|
| **C1** | the `bctrl` encoder | **stands**, and it is one afternoon. `work/w-ifn/price.py` script-counts it as the body's only missing mnemonic |
| **C2** | an indirect call as a `Selected` shape | **stands.** `coff::Call` carries a callee NAME and this call has none: `lwz r11,8(r31) · mtctr r11 · bctrl`, no REL24, no external symbol |
| **C3** | a bound call statement | **stands** |
| **C4** | a braceless early return on a call result, on `cr0` | **stands**, and there are TWO of them in this body (`+0x34`/`+0x38` and `+0x54`/`+0x58`), not one |
| **C5** | the elision and the volatile park | **stands as a mechanism, and its INPUT is paid by this lane.** Both facts are about `mmioFlush` and `mmioSetBuffer`, and both are now readable: the attribute above, plus the sibling's own body, which `TuContext` already carries |
| **C6** | *"the acceptance seam for C5 — there is nowhere in the port to ask it"* | **REFUTED.** §4 |
| **C7** | **the `.gl` NAME binding** — 10 of 11 names carry no `@@`, and two (`mmioSeek`, `mmioRead`) are exactly 8 characters and hit `INLINE_NAME_MAX` as well | **NEW, and it is the FIRST blocker.** Board **#1721** declined this widening with its reason: a TU with no mangled name anywhere comes back with `Bindings::unclaimed` **empty**, so `IlBundle::functions`' unclaimed-symbol gate goes **vacuous rather than satisfied** — on this TU |
| **C8** | **the whole-TU inline fence** — `mmioClose` calls `mmioFlush`, which this TU defines, so `bundle.rs`'s `callee_defined_here` refuses the TU wholesale | **NEW.** `w-ifn` counted neither this nor the composition fence, because neither is reachable while C7 stands |
| **C9** | **a REL24 against a symbol this obj DEFINES.** `bl mmioFlush` relocates against symbol \[33], `sec=10` — a defined function, not an undefined external. `Function::introduced_externals` mints undefined externals; nothing mints a self-relocation | **NEW**, read off `work/w-ifn/ref/mmio.dump.txt` |

**Nine, and only C1–C5 are the ones a codegen lane would build.** C7 gates
everything and is declined by a board row that predates this lane.

---

## 4. C6 — the architectural objection, refuted in code

`w-ifn` §4: *"Board **#139** puts acceptance in the PARSER, and the parser sees
exactly one `.ex` segment. There is no place in the port today where a sibling
function's body can gate parser acceptance."*

**`IlBundle::functions()` is that place, and it has been for its whole life.**
The conflation is between the *body* parser (`parse_segment`, one segment) and
the *acceptance* seam (`IlBundle::functions`, the whole bundle). #139's
invariant — the quantity the scan prints as
`fnbyte-census-disagree-expressible`, **target 0** — is that acceptance and the
census ask ONE question through ONE predicate. It says nothing about how many
segments that question may look at.

Four clauses of `functions()` already look at more than one, today, on master:

| clause | what it reads |
|---|---|
| `drectve_is_boilerplate(gl)` | the whole `.gl` |
| the label-counter gate | `funcs.iter().any(\|f\| f.is_framed())`, then every non-framed function's stride |
| the unclaimed-`.gl`-symbol accounting | every callee, data symbol and EH unwind callee of **every** function |
| **`callee_defined_here(f, &defined)`** | `defined` is built from **all** the names — the fence `w-inlfence` factored out precisely so it could be asked in three places about one bundle |

So a bundle-level pass that establishes a sibling fact before the per-function
loop and consumes it inside is not a violation of #139; **it is where #139's
acceptance already lives for exactly this kind of fact.** This lane does it:
`let attrs = super::gl::gl_function_attrs(gl);` sits above the loop in
`functions()` and above it in `census_functions()`, and both fill
`IlFunction::inlinable` from the name each function was bound by — the census
through `EmitBinding::name`, the gate through the per-record name, which is the
#918 discipline rather than a shortcut.

**What C6 got right, and it is worth keeping**: `elide.rs` resolves its
sibling question at *emit* time and is sound there because both of its outcomes
are valid objs, and that is not true of the elision or the park. Those must gate
acceptance. This lane's answer is that they *can*.

---

## 5. What this lane did NOT do, with sizes

* **It did not widen the `.gl` name binding (C7).** Board #1721's decline stands
  and its reason applies to this TU by name. Size: 10 of 11 records, plus a gate
  that goes vacuous.
* **It did not attempt `mmioClose` (C1–C5).** With C7 standing, a byte-exact
  `mmioClose` converts nothing, and `D1` refused byte work with no grade behind
  it. Size: `w-ifn`'s reader is 570 lines for a strictly simpler shape.
* **It did not narrow the PARSER's `callee_defined_here` (C8).** The bit makes
  that narrowing *available* — and it would be the same clause the composition
  fence now carries — but the TU it would convert is not this one and the lane
  measured no population for it. Registered as unminted board work.
