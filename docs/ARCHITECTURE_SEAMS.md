# ARCHITECTURE_SEAMS — restructuring c2-rs for concurrent agents

Status: plan, written 2026-07-30 against master `9ec4871`. Nothing here is
implemented; every step below is sized, ordered, and gated by the existing
differential. The constraints of `CLAUDE.md` bound everything in this
document: std only / zero external crates (so every "registry" below is a
hand-rolled match or a const table, never a macro crate); real `c2.dll` under
wibo + byte-exact obj compare is the sole judge; acceptance fails closed
(`NotImplemented`, never a guess); the CLI degrades cleanly without the
toolchain.

**The problem being solved.** On 2026-07-30, ~9 rungs landed in parallel
worktrees with a serial merge funnel. It worked (census 140,476 → 474,103;
12 live mis-emits found and fixed; mismatch 0 throughout), but the collisions
were of five distinct classes, and four of them are *structural* — they will
recur on every future rung until the structure changes:

1. a 231-line conflict in `bundle.rs`'s symbol-binding seam, where a wrong
   resolution is **silent** (roadmap #14's defect class);
2. `c2-core/src/codegen.rs` contended all day — it holds every lowering;
3. `body/shapes.rs`, whose conflict hunks share closing braces and whose one
   giant `mod tests` every rung appends to;
4. semantic conflicts git never flagged: a duplicate `encode_std`, a new
   `IlFunction` field missing at the other branch's constructor sites, the
   rung tag `W23` claimed twice, doc sections §6e/§6f/§6g/§6i each claimed
   twice;
5. append-only central files (`scripts/expr_sweep.sh`, `docs/ROADMAP.md`,
   `docs/GAPS.md`) conflicting on every single merge.

The design goal is **not** maximum parallelism. It is: every collision that
is merely *file-shaped* (two rungs touching one big file about unrelated
facts) becomes a new-file addition; every collision that is *fact-shaped*
(two rungs about the same fact) stays visible and lands on one named module;
and the genuinely serial work (the frame/liveness spine) is named as serial
instead of being parallelized into wrong bytes.

---

## 1. Where the true seams are

### 1.1 The contention map, measured

| file | lines | of which tests | contended today? | verdict |
|---|---:|---:|---|---|
| `crates/c2-core/src/codegen.rs` | 4,426 | ~1,747 (`mod tests`, 2501–4248) | **yes, repeatedly** | big file, mostly *uncoupled* — split pays |
| `crates/c2-il/src/func/body/shapes.rs` | 4,067 | ~1,057 (3010–4067) | **yes** | big file, mostly uncoupled — split pays |
| `crates/c2-il/src/func/bundle.rs` | 856 | ~46 | **yes (231-line conflict)** | *coupled*, but wrongly shaped — restructure, don't just split |
| `crates/c2-il/src/func/body/mcall.rs` | 3,465 | — | no | big **and** coupled (one classification walk) — leave |
| `crates/c2-il/src/func/body/mod.rs` | 1,399 | ~600 | edited per rung (1 line + keys) | central dispatch — keep, make edits one-line |
| `crates/c2-core/src/coff.rs` | 1,676 | ~ | yes (spine agents) | coupled, owned by the serial spine — leave |
| `crates/c2-il/src/codec.rs` | 2,156 | — | no | independent lane (roadmap #14/codec) — leave |
| `crates/c2-harness/src/*` | ~9,600 | — | no | per-concern files already — leave |
| `scripts/expr_sweep.sh` | 1,376 | — | **every rung** | append-only monolith — fragment |
| `docs/ROADMAP.md` / `docs/GAPS.md` | 2,361 / 1,907 | — | **every merge** | append-only monoliths — freeze + per-rung files |

### 1.2 "Big file" vs "genuinely coupled", per hotspot

**`codegen.rs` is five modules wearing one filename.** Reading it top to
bottom: (a) ~50 pure `encode_*` word encoders with zero dependencies on
anything; (b) five leaf lowerings (`indirect_load_text`, `addr_leaf_text`,
`store_leaf_text`, `float_leaf_text`, `compare_leaf_text`), each a
self-contained pattern-match over an exact op stream, each independent of the
others, sharing only the encoders and two helpers (`out_of_class`,
`fits_i16`); (c) the straight-line selector (`select_text`, `combine`, the
`Plan`/`Base`/`Operand` machinery, `try_select_depth2_tree`) — genuinely
coupled *internally*, one unit; (d) the frame/call complex (`FrameLayout`,
`FramedBody`, `SeqBody`, `framed_call_text`, `call_seq_text`/`call_seq_parts`,
`ops_setup_text`, `int_tail_call_text`, `permute_args_text`) — genuinely
coupled to each other and to `coff.rs`, and it is the serial spine's home;
(e) the dispatch (`Selected`, `select_function`, `function_gate`,
`opt_mode_of_word`) — ~180 lines, the only part where *every* rung's edit is
forced to touch the same lines, and that edit is ~5 lines per rung.

Two agents wanting `codegen.rs` simultaneously were almost never wanting the
same *fact* — one was in a leaf lowering and one in the frame path. That is
file-shaped contention, and a mechanical split removes it. The exception is
(d): two agents in the frame/call complex genuinely conflict, and should —
that work is serial (§8).

**`shapes.rs` is the same shape.** The `try_parse_*` recognizers are
deliberately non-committal (cursor copy, `Option` return, no side effects),
so they are independent by construction. The genuinely shared facts inside
the file are small and nameable: the sub-object designator
(`parse_base_member_designator`, `eat_addr_offset_adds`, `sized_ptee`,
`sized_ptr_width`, `is_ptr_any`, `store_value_width`) — one fact with three
graded consumers (load/addr/store leaves) plus the dtor member receiver; the
`this` binding (`ThisBinding`, `parse_this_token`, `read_this_group` — GAPS
§6 instances #1/#2 live here); and the unified call-shape machinery
(`tail_call_shape`, `eat_call_head`, `eat_call_args`, `parse_call_sequence`,
`permutation_cycles` — instance #9 was closed by making this ONE copy). The
shared-closing-brace conflict class is a direct artifact of unrelated
recognizers being adjacent in one file; per-shape files end at a file
boundary and the class disappears.

**`bundle.rs` is coupled, but the conflict was not about the coupling.** The
file holds two different things: (i) the *correspondence seam* — binding
`.gl` defined-name records to `.ex` segments positionally with a fail-closed
1:1 offset check, the `GlIndex` token→symbol resolution, the `.sy` exit-label
keying — which is roadmap #14's defect class, where a wrong resolution is
silent because the oracle cannot grade a correspondence (`GAPS.md` §6, the
`.sy` bullet); and (ii) `shape_to_function`, a 12-arm match in which **every
arm spells out all eleven `IlFunction` fields**. The 231-line conflict and
the silent `call_seq`-missing-at-constructor-sites conflict both came from
(ii), which is pure boilerplate: every arm sets its own two or three fields
and `None`/`false` for everything else, so any two rungs that each add a
field or an arm collide across the whole match. (i) is load-bearing and
small; (ii) is noise that the type system is currently making expensive.
`func/mod.rs` itself already says the parallel `Option` fields "want to be
one enum" — the doc block on `IlFunction::empty_body` — and it is right; §2.3
takes the cheap half now and defers the sum type to the W8 IR restructure,
as roadmap §G4 already schedules.

**`mcall.rs` (3,465 lines) is big and actually coupled** — one backward
classification walk with shared state, built explicitly around the
sharded-key and mis-attribution lessons. It accepts nothing and is edited by
one lane (census/decode). Splitting it would manufacture interfaces inside a
single algorithm. Leave it.

**`coff.rs` is coupled and spine-owned.** §6e already deleted the third
emitter because "one rule implemented in two emitters and fixed in one" had
caused two bugs. The remaining two emitters (`emit_obj`, `emit_comdat_obj`)
plus `plan_labels` share the symbol-order and label-stride facts that every
framed rung touches. Splitting it would spread one fact across files —
exactly the defect class this repo keeps finding. Leave it whole, owned by
the spine lane.

**The harness is already fine.** `gap.rs`, `capture_cache.rs`,
`provenance.rs`, `census` are per-concern files; no contention was reported.
`search.rs`/`main.rs` are big but single-lane.

---

## 2. The decomposition, concretely

### 2.1 `c2-core/src/codegen.rs` → `c2-core/src/codegen/`

```
codegen/
  mod.rs           pub-use re-exports of every current public name, so
                   `c2_core::codegen::encode_add` etc. keep working and the
                   split is diffable as a pure move. Holds nothing else.
  encode.rs        every encode_* word encoder + fp_primary/fp_a_form/xo31.
                   Pure functions, no deps. This is where duplicate-encoder
                   conflicts (the two `encode_std`s) go to die: one file whose
                   whole content is alphabetizable one-liners, where a
                   duplicate is a compile error in the SAME file rather than
                   two definitions 2,000 lines apart.
  select.rs        OptMode, Selected, select_function, function_gate,
                   opt_mode_of_word, out_of_class, fits_i16. The one shared
                   touch point, kept deliberately thin (§3.1).
  straightline.rs  select_text, combine, Plan/Base/Operand/PlanOp,
                   try_select_depth2_tree, emit_add_imm, emit_load_imm.
                   One unit; the r11→r9 cursor and the depth-2 tree rule
                   live and die together.
  leaf/
    mod.rs         nothing but `pub mod` lines.
    load.rs        indirect_load_text          (+ its tests)
    addr.rs        addr_leaf_text              (+ its tests)
    store.rs       store_leaf_text             (+ its tests)
    float.rs       float_leaf_text, FpConstRef (+ its tests)
    compare.rs     compare_leaf_text           (+ its tests)
  frame.rs         FrameLayout (+ prologue()/epilogue()), the frame-size
                   arithmetic, probe_pages/needs_* predicates, and the
                   FrameLayout cross-check assertions from
                   CODEGEN_FRAMED_CALLS.md §1.2.
  calls.rs         FramedBody, SeqBody, framed_call_text, call_seq_text,
                   call_seq_parts, ops_setup_text, int_tail_call_text,
                   permute_args_text, encode_tail_branch/encode_call_branch.
                   ONE module on purpose: this is the serial spine's home
                   (multi-call, values-live-across-calls, Class B/C), and
                   pretending its parts are independent would invite two
                   agents to guess allocator order concurrently.
```

The ~1,747-line `mod tests` is split with the code it tests: encoder word
tests into `encode.rs`, each leaf's tests into its leaf file, frame tests
into `frame.rs`, and the cross-cutting selector tests into `select.rs`.
After this, a new leaf rung touches: its own new `leaf/<shape>.rs`, one arm
in `select.rs`, one `pub mod` line — and nothing else in `c2-core`.

**Interface between modules:** unchanged from today's function signatures —
`fn <shape>_text(func: &IlFunction, …) -> Option<Result<Vec<u8>, BackendError>>`
for the try-style leaves, `Result` for the committed ones. The point of the
split is that these signatures already *are* clean seams; they are just
hidden in one file. No new abstraction is introduced, so the refactor is
verifiable as byte-identical output (§6).

### 2.2 `c2-il/src/func/body/shapes.rs` → `body/shapes/`

```
body/shapes/
  mod.rs           pub(crate)-use re-exports; nothing else.
  designator.rs    the sub-object designator: parse_base_member_designator,
                   eat_addr_offset_adds, sized_ptee, sized_ptr_width,
                   is_ptr_any, store_value_width. Doc header names its
                   consumers (load leaf, addr leaf, store leaf, dtor member
                   receiver). ONE locator for the fact "how a byte offset
                   into an object is spelled in IL".
  this_binding.rs  ThisBinding, parse_this_token, read_this_group. The
                   line-70 lesson lives in its doc header.
  ctor_dtor.rs     eat_ctor_this_epilogue, try_parse_empty_dtor_delegation,
                   eat_dtor_base_receiver, eat_dtor_member_receiver.
  leaf_load.rs     try_parse_indirect_load_leaf, try_parse_ptr_identity_leaf,
                   finish_indirect_load{,_of} (shared by exactly these two).
  leaf_addr.rs     try_parse_addr_leaf.
  leaf_store.rs    try_parse_store_leaf.
  calls.rs         tail_call_shape (the ONE copy, instance #9),
                   eat_call_head, eat_call_args, arg_loads_are_formals,
                   permutation_cycles, parse_call_sequence.
  testutil.rs      #[cfg(test)] transcript builders and segment helpers the
                   per-shape test modules share today inside the one
                   mod tests.
```

Each file carries its own `mod tests`. A new rung's tests are a new file (or
an append to *its own shape's* file), never an append to a 1,057-line shared
module — which also removes the "two sides share a closing brace" conflict,
because the brace being fought over was the shared `mod tests`'.

`body/mod.rs` stays the owner of `BodyShape`, `Block`/`feature`, and the
`parse_segment_shape` dispatch. It shrinks (the recognizers' bodies leave)
but is not split: the dispatch order is load-bearing and must stay readable
in one screen (§3.2).

### 2.3 `bundle.rs`: name the binding seam, defuse the flatten

Two moves, different risk profiles:

**(a) Extract `func/bind.rs` — the correspondence seam, with an explicit,
testable interface.** Moves: `gl_defined_names` + the offsets-must-equal-
split-points check, `GlIndex` (token→symbol, with the injectivity
third-value drop), `SyLocals` (exit-label keying), `mangled_is_varargs` and
the name-derived gates. Interface:

```rust
pub(crate) struct Bindings<'a> {
    /// Defined-function names, 1:1 with .ex segments, or None (fail closed).
    pub names: Option<Vec<String>>,
    /// Token → mangled symbol, injective by construction (conflicts dropped).
    pub resolve: GlIndex<'a>,
    /// Per-segment locals, keyed by exit label; empty unless .sy parsed whole.
    pub locals: SyLocals,
}
impl<'a> Bindings<'a> {
    pub(crate) fn of(bundle: &'a IlBundle, segs: &[&[u8]], starts: &[usize]) -> Bindings<'a>;
}
```

built once, consumed by both `functions()` and `census_functions()` — so the
two callers *cannot* bind differently, which is the two-locators defect
pre-empted structurally. The module's unit tests are the binding invariants
that already exist scattered (injectivity; exit-label monotonicity; the
1:1-with-offsets check; the dtor-callee-must-be-`??1` class check), plus the
two standing counters that print on every `gap`/`census` run stay exactly
where they are. This module is the *right* place for roadmap #14's follow-up
(the census positional-name fix — `census_functions` zipping
`mangled_names` onto segments — is a `bind.rs` change with `gl_defined_names`
as the one locator).

**(b) Collapse `shape_to_function`'s boilerplate with a base constructor +
struct-update syntax.** Add:

```rust
impl IlFunction {
    /// Everything a shape does NOT discriminate on: provenance + "no shape".
    fn base(name: &str, src: &Option<String>) -> IlFunction { … all-None/false … }
}
```

and each arm becomes only the fields it owns:

```rust
BodyShape::StoreLeaf { params, ops } =>
    Some(IlFunction { params, ops, ..IlFunction::base(name, src) }),
```

This turns "new `IlFunction` field" from *edit 12 arms, conflict with every
in-flight branch, and silently miss the arms a concurrent branch is adding*
into *edit `base()` once*. The honest trade-off: with `..base`, the compiler
no longer forces every arm to consider a new field. That is acceptable here
and only here, because for shape-discriminant fields the correct value in
every other arm **is** the default (`None`) — today's arms say so explicitly
eleven times over — and the case where a default is wrong is exactly what
the census/gate cross-check and the fixture N/N discipline grade (§4). The
principled fix — `IlFunction.body: BodyShape`, deleting both this match and
`select_function`'s re-derivation — is real, is already argued for in
`func/mod.rs`'s own doc comment, and is **deferred to the W8 IR restructure**
(roadmap §G4 schedules the restructure there; doing it now rewrites
`codegen.rs` wholesale while two agents are inside it).

### 2.4 `scripts/expr_sweep.sh` → `scripts/sweep.d/` fragments

```
scripts/expr_sweep.sh          driver: builds c2rs, iterates scripts/sweep.d/,
                               grades, reports per-fragment and total counts.
scripts/sweep.d/
  10-int-chains.py             def cases(emit): …   # today's §1
  20-compare.py                # W6 grid
  30-fp-leaves.py              # FP operand/operator sweep
  31-fp-params.py              # the parameter-list axis
  40-framed.py                 # W-UNW cases
  …one file per axis; a rung adds a NEW file.
```

Contract per fragment: it defines `cases(emit)` where `emit(src)` is
provided by the driver; the driver namespaces output files by fragment
(`10-int-chains-0007.cpp`), so **no fragment can touch another fragment's
counter** — the `n`-shadowing trap that silently dropped 1,679 cases becomes
unrepresentable, not merely fixed. The driver prints
`fragment 31-fp-params: 96 cases` per fragment and **fails if any fragment
emits zero cases** (the observable symptom of the shadowing bug, now a hard
error). Two rungs adding fragments never conflict; two rungs claiming the
same fragment *name* is an add/add conflict git flags loudly.

### 2.5 Docs: freeze the monoliths, per-rung files, generated index

```
docs/rungs/
  2026-07-30-store-leaf.md     one file per rung, standard header:
  2026-07-30-call-seq.md         Tag / Fixtures / Census before→after (+Δ)
  …                              / Estimate-vs-outcome / Gate table /
                                 Found-and-not-taken
docs/rungs/INDEX.md            one line per rung, regenerated by
                               scripts/gen_rung_index.sh (ls + head parse;
                               tooling, like plot_perf.py, outside the
                               std-only workspace)
```

`ROADMAP.md` keeps §1–§7 (strategy, invariants, the ladder) and **stops
growing §6-letter sections**: the historical §6a–§6j stay frozen where
references point, with a one-line note that new rungs land in `docs/rungs/`.
`GAPS.md` likewise: §6's instrument-failure log is the most load-bearing
prose in the repo and its existing entries stay put; *new* instances become
`docs/gaps.d/NN-<slug>.md` files listed by an index, so two rungs each
logging an instrument failure no longer serialize on one file. Section
*numbers* stop being allocated by authors entirely — a rung doc is named by
date+slug, and nothing needs a §-number again. This costs some narrative
linearity (ROADMAP currently reads top-to-bottom as a ledger); the standard
header and the index are the mitigation, and the per-rung format is already
the de-facto template every §6-letter section uses.

---

## 3. Making rungs additive instead of edits

The recurring rung shape is "add an accepted function class", which today
edits: `select_function`, `parse_segment_shape`, a giant test module, the
sweep script, and two docs. After §2, the last three are new files. The
first two stay central **on purpose**, but shrink to one line each:

### 3.1 `select_function`: keep the explicit ordered match

Each arm delegates to a per-module lowering with today's uniform try-shape:

```rust
if let Some(t) = leaf::store::lower(func)   { return Ok(Selected::Plain(t?)); }
```

**Deliberately rejected: a registration table** (a
`const LOWERINGS: &[fn(&IlFunction, OptMode) -> Option<…>]` slice, or any
hand-rolled registry). Std-only makes it buildable, but the dispatch *order*
is load-bearing and documented (framed before tail before leaves, compare
before float before load before identity before addr — `select_function`'s
doc comment and `parse_segment_shape`'s both say so, with reasons per
adjacency). A table hides order behind data; a match keeps it in the one
place a reviewer reads. The conflict profile of one-line adjacent inserts is
also *safe*, not just small: recognizers and leaf lowerings are
non-committal, so a bad merge that **duplicates** a line is harmless (the
second probe never fires), and a bad merge that **drops** a line un-accepts
a shape — which the rung's own positive fixture catches, because `c2rs
bench` requires it to be `Port=Match` and `census N/N` (§5.3 makes that
fixture mandatory). A wrong *order* after a merge is the residual risk; it
is the same risk today, and the sole-judge gate catches it only where shapes
overlap — which is why every ordering in those two matches must keep its
one-line "why this adjacency" comment attached to the arm it orders.

### 3.2 `parse_segment_shape`: same treatment

The `0xB9 | 0x33` chain already is a one-line-per-recognizer ladder; after
§2.2 the bodies live in per-shape files and the ladder is the whole edit. A
new statement-opening shape adds one match arm; a new expression-opening
leaf adds one `if let Some(shape) = shapes::leaf_x::try_parse(…)` line.

### 3.3 Census keys stay decentralized

`Block { ctx: &'static str }` keys are declared at their refusal sites. Two
rungs claiming the same key string would merge two buckets — visible in the
histogram, mild, and not worth a registry test. (Flagged as not-worth-doing
in §9.)

### 3.4 Rung tags: slugs claimed by filename, numbers assigned at merge

The `W23`-claimed-twice collision came from agents allocating small integers
concurrently. Mechanism: a rung's identity is its **slug** (=`docs/rungs/`
filename = fixture prefix, e.g. `w25_store_leaf` → slug `store-leaf`), which
collides as a git add/add conflict instead of silently. If W-numbers are
kept at all, the merge funnel assigns the next free number at merge time —
the one serial actor allocating from the one sequence. A portable-lane test
(new file, `crates/c2-harness/tests/rung_registry.rs`, std-only directory
walk, no toolchain needed) asserts: no two fixtures claim the same `wNN`
prefix; every `wNN` prefix that exists in `fixtures/cpp` is referenced by
exactly one `docs/rungs/*.md`. This is the "fail if two shapes register the
same tag" check, and it also catches the doc-section double-claim class,
because doc files are the registry.

---

## 4. Preserving the invariants that caught this session's bugs

**Acceptance stays in the IL parser — nothing in this plan moves a gate.**
The split of `shapes.rs` keeps every `try_parse_*` and every out-of-class
refusal in `c2-il`; the split of `codegen.rs` keeps `function_gate` running
`select_function` itself (not a copy), and `tests/census_gate.rs` continues
to pin the per-lane disagreement at its recorded values (1 packed / 9
`/Gy`, causes named). The migration acceptance criterion for every step in
§6 includes: census numerator, per-key histogram, and both disagreement
counters **byte-identical before and after** — a mechanical split has no
license to move any of them by 1.

**One locator per fact gets a structural assist, not just a norm.** The
signature bug of the session (mis-emit #11: one rule, two copies, each
missing a gate the other had) is *harder to write* after §2 for three
reasons, stated honestly with their limits:

1. **Shared facts become named modules with named consumers**
   (`designator.rs`, `this_binding.rs`, `bind.rs`, `encode.rs`,
   `shapes/calls.rs`). Inside a 4,000-line file, the second copy of a rule
   is invisible — you grep, miss one spelling, and paste. When the fact is a
   module, the import is the tell: a rung file that *doesn't* import
   `designator` but parses an offset chain is visibly reinventing it in
   review. This is a review affordance, not an enforcement; the enforcement
   remains the census/gate cross-check and adversarial probing.
2. **The flatten defuse (§2.3b) removes the strongest *pressure* to copy.**
   Instance #9 happened because the direct and bound forms each carried
   their own argument validation; the unified `tail_call_shape` is kept as
   one module that both — and the future statement-call forms — import.
3. **The split itself is the one moment the defect class is easy to
   commit** (copying a helper into two destination files instead of moving
   it). Mitigation is mechanical: after each split step, every moved symbol
   must have exactly one definition (`grep -rn "fn parse_base_member_designator"`
   count = 1), asserted in the migration commit message with the actual
   counts. Duplicate *definitions* in one crate are compile errors; the
   dangerous case is a private copy under a new name, which only the
   one-definition grep discipline and review catch.

**The whole differential gate stays runnable and cheap.** Nothing in the
plan touches `c2-reference`, the capture path, or the gate commands in
`GAPS.md` §6. The capture cache (36.5 s → 0.9 s warm) is what makes the
migration plan viable at all: every step, however mechanical, gets the
*full* gate (workspace tests, `bench`, four mode lanes, the sweep, the
878-TU scan with census + disagreement compared to the function) for about a
minute of wall clock, not forty.

---

## 5. Merge hygiene, beyond the file layout

1. **Per-rung fixtures are the merge safety net — make the norm a check.**
   Today's practice (every rung lands `wNN_<slug>.cpp` positive +
   `wNN_<slug>_neg.cpp` negative, graded N/N and 0/N) is what makes
   one-line dispatch merges safe (§3.1). The `rung_registry.rs` test adds:
   every `docs/rungs/*.md` names at least one fixture that exists. The
   existing lesson "a positive case sharing a TU with a refused one is
   decoration" (`GAPS.md` §6) stays a review rule; it cannot be a cheap
   static check.
2. **Sweep fragments fail loudly on zero cases** (§2.4) — the class of "the
   script silently dropped 1,679 cases" becomes an error, and per-fragment
   counts make the drop attributable when it happens anyway.
3. **JSONL/measurement hygiene** is already handled (provenance record 0,
   absolute-path rule after the false-correction incident in ROADMAP §6);
   this plan adds nothing there and deliberately so.
4. **Branch-scoped naming everywhere a name is claimed:** fixture prefixes,
   sweep fragment names, rung doc filenames, census keys. All become
   filesystem-visible claims (add/add conflicts) except census keys (§3.3,
   accepted).
5. **The serial merge funnel stays.** One serial actor re-running the full
   gate per merged tree is not a bottleneck worth removing at ~1 minute per
   merge, and it is the only place cross-branch *semantic* interaction is
   measured (the W25/W26 merge measured its interaction term as exactly 0 —
   that discipline survives only if merges stay serial).

---

## 6. Migration sequence

Every step is independently landable, verified by the full gate, and leaves
master green. "Quiesce" means: no agent holds an unmerged branch touching
the file being split — a mechanical 4,000-line move conflicts with
*everything* in flight, so it lands in a gap between rung waves, announced
in advance. With the warm cache, each step's verification is minutes.

| # | step | quiesce? | est. collision reduction | risk & how the gate catches it |
|---|---|---|---|---|
| 0 | **Sweep fragmentation** (§2.4) + **rung-doc convention + `rung_registry.rs`** (§2.5, §3.4) | no — new files + one script rewrite nobody is editing mid-rung | removes 2 of the 5 conflict classes outright: the sweep conflict (every rung) and the doc-section/tag conflicts (every merge). Highest ratio in the plan. | sweep rewrite could drop cases: driver prints per-fragment counts and the *total must equal the old script's 5,023* on the transition commit; a fragment emitting 0 fails. |
| 1 | **Split `codegen.rs` → `codegen/`** (§2.1), re-exports in `mod.rs`, tests distributed | **yes** (~half a day incl. gate) | the single most-contended file stops being one file; leaf rungs and spine rungs stop colliding at all in `c2-core`. Guess: eliminates most of the day's repeated `codegen.rs` contention. | a mis-moved or truncated function, or an accidental dispatch reorder → byte-different objs → `bench`/mode lanes/sweep catch it; census + disagreement must be identical to the function. One-definition grep per moved symbol (§4.3). |
| 2 | **Split `shapes.rs` → `body/shapes/`** (§2.2) with per-shape tests + `testutil.rs` | **yes** (~half a day) | removes the shapes.rs conflict class incl. the shared-brace failure; new-shape rungs become new-file rungs in `c2-il` too. | same recipe as step 1; additionally the census histogram (569-ish keys) must be key-for-key identical — a recognizer lost in the move shows up as a key's count moving. |
| 3 | **`bundle.rs`: extract `bind.rs`** (§2.3a) **and the `..base` flatten defuse** (§2.3b) | preferable (file is small; a quiet moment suffices) | removes the 231-line-conflict class and the missing-constructor-field class. | the binding seam is wrong-but-green territory: the extraction must be move-only, graded by (i) scan JSONL rows byte-identical, (ii) the standing binding counters (dup-token count, dtor `??1` class) unchanged, (iii) `bind.rs` unit tests transplanted intact. The `..base` change is behavior-identical by construction; census/gate cross-check pins it. |
| 4 | **Retro-index docs**: freeze ROADMAP §6-letters and GAPS §6 with pointer notes; generate `docs/rungs/INDEX.md` | no | small by itself; completes step 0. | none — prose only. Do not renumber or move existing sections; too many cross-references point at them. |
| 5 | **`IlFunction.body: BodyShape` + real block IR** | **yes, and scheduled with W8**, not before | removes `select_function`'s re-derivation tree entirely; the last structural home of "two decision trees over one fact". | this is the G4 restructure; it rewrites both crates' cores and must own a quiesce window of its own, with the full gate at every intermediate commit. Not part of this plan's near-term sequence — listed so nobody half-does it early. |

Steps 0 and 4 are safe with agents running. Steps 1–3 are the "nobody in
flight" steps, and they should be done in **one announced window, in that
order** (1 and 2 are independent of each other but both conflict with
everything; doing them back-to-back costs one window instead of two).
Estimated total quiesce: one working day. All reduction estimates above are
informed guesses from one day's collision log — the honest statement is
that steps 0–3 each remove a conflict class that *did occur* today, and
none of them can reintroduce a defect silently if the gate discipline in
§4 is followed.

---

## 7. How many agents, on what partition

Mapped against the actual remaining work (ROADMAP §6b's distance list, §6j's
handoff, §6a's frame audit), after steps 0–3:

| lane | work | files owned | agents |
|---|---|---|---|
| **The spine — serial, irreducibly** | Class B (values live across calls, r31, the liveness answer), then Class C helpers **with** the 7-not-5 `/Gy` stride landing together, multi-call accumulators, then EH records (`/EHsc` is the whole workload) | `codegen/calls.rs`, `codegen/frame.rs`, `coff.rs`, `shapes/calls.rs` | **1**. Not 2. Each rung is byte-exact on the previous one's model; the register-assignment order is measured-not-explained past n=2, and two agents extending it concurrently would resolve disagreements by guessing — the one thing the doctrine forbids. The spine is the critical path and stays serial *by design*. |
| **FP/leaf lane** | the FP tail-call rung (85,231 functions), `lbz`/`lfs`/`lfd` getter tails, remaining leaf shapes from the one-away lists | `codegen/leaf/float.rs` (+ a new `shapes/` file per shape), `leaf/*.rs` | 1–2 |
| **Decode/census lane** | intrinsic family decode → per-id allow-list acceptance, `codec.rs` variable-width port (roadmap #14-adjacent, its own file), varint/type-width residue, `mcall.rs` refinements | `mcall.rs`, `codec.rs`, `readers.rs`, own shapes files | 1–2 |
| **Control flow (W8)** | the `.ex` branch/label grammar (`body-0x29`, `expr-op-0x3A`) — the **decode half is concurrent-safe** in its own shapes/census files; the **codegen half forces the block IR** and is step 5's quiesce, sequenced behind or with the spine | shapes/ + docs first | 1 (decode now; lowering waits) |
| **Ground truth / characterization** | capture campaigns (docs only): FPR-helper stride, `.pdata`-beside-`.rdata`, the 2+-FP-constant scheduler, opt-word `00200001`, permutation order past 3-cycles | `docs/*`, `work/` probes | 1–2, zero collision by construction |
| **Instrument/harness** | census key checks, cache validation, scan tooling | `c2-harness/src/*` | 1 |
| **Front end (Track D)** | P-F0.2 probes → `c1-core` crate | new crate | 1, zero collision |

That is **6–8 concurrently productive agents** against today's effective
~2–3 (the day's nine rungs landed, but with two agents blocked on
`codegen.rs` "most of the day" and every merge paying doc/sweep conflict
tax). The binding constraints after the restructure are the serial spine
and the serial merge funnel — both serial on purpose — not file contention.
That number is a guess; the defensible claim is the shape, not the count:
leaf, decode, ground-truth, instrument and FE lanes stop sharing any file
with each other or with the spine.

---

## 8. What could go wrong, stated per proposal

- **The splits (steps 1–2)** can silently duplicate a helper (§4.3), change
  dispatch order, or lose a recognizer. All three are byte-visible to the
  existing gate *except* a duplicated private helper, which is caught only
  by the one-definition grep and review — named as the residual risk.
- **`bind.rs` (step 3)** is the one step in wrong-but-green territory: a
  botched move of the correspondence code could bind names differently and
  still scan green (the oracle cannot grade a correspondence). Mitigations
  in §6's table row; the deep one is that the move is a *move*, with the
  binding's own invariant counters — which exist precisely because of this
  failure mode — compared before/after on the same corpus HEAD.
- **`..base` struct-update** trades compiler-forced field review for merge
  immunity; the trade is justified in §2.3b and bounded by the census/gate
  cross-check. If a future field ever has a non-`None` correct default per
  shape, it must not go through `base()` — put that sentence in `base()`'s
  doc comment.
- **Doc fragmentation** costs narrative linearity; if `docs/rungs/` decays
  into unread files, the fix is the index and the standard header, not a
  return to the monolith.
- **The one-line dispatch edits** still collide (adjacent inserts). That is
  accepted: the conflict is trivial, and its wrong resolutions are covered
  by the mandatory positive fixture (drop) and non-committal recognizers
  (duplicate). Order mistakes remain reviewable-only — keep the per-arm
  ordering comments.

## 9. Deliberately rejected

1. **A runtime/registry dispatch for shapes or lowerings** — order is
   load-bearing, std-only makes registries pure ceremony, and the central
   edit it would remove is one line (§3.1). Rejected as negative value.
2. **Splitting `coff.rs`** — one fact (symbol order, label strides) per
   §6e's own hard-won consolidation; spine-owned; splitting it mid-spine is
   the worst merge in the repo. Rejected for now; revisit if a data-section
   lane (W14) ever needs to share it, and then split by *section kind*, not
   by emitter.
3. **Splitting `mcall.rs`** — big but genuinely one algorithm with shared
   state and carefully centralized key discipline. Rejected.
4. **Splitting `search.rs` / `main.rs`** — no measured contention;
   single-lane harness work. Not worth the churn.
5. **A census-key registry/uniqueness test** — a key collision merges two
   histogram buckets, which is visible and mild; the test would be another
   central file. Rejected.
6. **Retro-converting ROADMAP/GAPS history into fragments** — history does
   not conflict; only growth does. Freeze and fork (§2.5), don't migrate.
7. **Doing the `BodyShape`-in-`IlFunction` sum type now** — right change,
   wrong moment; it rewrites both crates' cores under two active agents.
   Scheduled with W8 (step 5), where the block IR forces the rewrite anyway.
8. **Parallelizing the spine** — two agents on the liveness/allocator model
   would produce wrong bytes to merge, and the doctrine has no way to grade
   "half a register-assignment rule". The spine is serial because the
   *evidence* is serial: each rung's captures only exist once the previous
   rung's model is byte-exact. Stated plainly rather than worked around.
