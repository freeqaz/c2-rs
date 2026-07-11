# K3a IL edit model — the length-consistent `.ex` edit primitive

What turns the K1/K2a lossless *read* codec (`crates/c2-il/src/codec.rs`) into a
verified *edit* substrate: an [`IlModel`] mutation that changes a function's `.ex`
operand-stream byte length and re-emits the one field the change obligates, so
c2 still consumes the bundle and returns the byte-exact obj the edited IL denotes.
This is the primitive every length-changing IL-space search move (T-A) routes
through.

The doctrine is unchanged from the rest of the port: **the compiler + objdiff is
the sole judge.** An edit "works" only when standalone c2 consumes the edited
bundle and the obj is byte-exact (timestamp-normalized) to a native capture of
the equivalent source — see the differential gate below. There is no fuzzy
fallback.

## What is length-plastic, and the one obligation

Proven live in `il-witness` P0.6a (reproducer `probes/p0_6a_length_plasticity.py`,
readout `../decomp-synth/docs/plans/il-witness/P0_6A_LENGTH_PLASTICITY.md`): the
`.ex` operand stream is **length-plastic**. Grow or shrink it and c2 re-optimizes
the result byte-exact to a native capture of the equivalent source — under ONE
obligation:

> On any `.ex` length change, every function *after* the edit point has its `.gl`
> body-start offset (`80 <LE32>` = the `.ex` byte offset of that function's
> `4F 1F` marker) bumped by the byte delta. The edited and preceding functions
> are unchanged. A single-function / last-function edit needs **no** `.gl` patch
> at all (the zero-bookkeeping regime).

The offset table lives in `.gl`, not the `.ex` header (the header is
length-insensitive). Skip the re-emit on a non-last edit and c2 seeks a
downstream function at a stale offset — P0.6a experiment C observed a SIGSEGV;
this port's gate additionally observed a *wrong obj* on a tiny mvp_lit widen
(same conclusion: the re-emit is load-bearing, the failure mode is
toolchain/shape-dependent). A `.gl`/`.ex` **function-set** mismatch (whole-fn
add/remove without coordinated record removal) can make c2 **hang** (P0.6a G), so
every replay is TIMEOUT-bounded.

## The edit API (`IlModel`, additive over K1/K2a)

All edits are statement-grain and confined to **one function's body**:

- `splice_function_tokens(fn_index, range, replacement)` — the primitive. Replace
  the typed [`ExToken`]s `[range]` of function `fn_index`'s operand stream with
  `replacement`. Insert = empty `range`; delete = empty `replacement`; substitute
  = both non-empty. Models P0.6a **E** (grow: insert `Lit; Add` → `(a+5)+5`) and
  **F** (shrink: delete `Load; Add` → `a+b`).
- `set_literal_wide(fn_index, token_index, wide)` — widen/narrow an int literal's
  varint form (same value, pure length change; P0.6a **A/B**). Built on the
  primitive. Narrowing a value outside `0..=0x7F` is refused (would change it).
- `function_tokens(fn_index)` — the function's typed tokens, in stream order, so a
  caller locates the splice/widen point by predicate (never a hardcoded index).

Each successful edit returns an `EditReport { fn_index, byte_delta, gl_offsets }`
(`gl_offsets` = the re-emitted offset column, empty iff none was needed).

### How the `.gl` re-emit works

After the token splice, the codec re-encodes the candidate `.ex`, re-derives the
`4F 1F` marker positions, and rewrites **every typed `Span::GlOffset` from those
new positions** — which leaves functions ≤ `fn_index` unchanged and bumps every
later function by the delta, exactly discharging the obligation. `GlOffset` was
typed, structurally located, and gated 1:1 with the function count back in K2a, so
this is a `u32` column rewrite, not a byte hunt.

### The fail-closed boundary (what is refused, and why)

Everything below returns a typed `EditError` and leaves the model **untouched** —
it never emits a hang/crash-inducing bundle:

- `FunctionSetChanged` / `DownstreamOffsetDesync` — the edit created or destroyed a
  `4F 1F` marker (whole-function add/remove), or a downstream start did not shift
  by exactly the delta. **Whole-function add/remove is OUT of K3a scope**: it needs
  coordinated `.gl` *record* framing and the `.sy` record (K3b), and its violation
  can hang c2 (P0.6a G), so it is a refusal, not a byte-poke.
- `GlOffsetsNotTyped` — a non-last edit whose `.gl` offset column is not modeled
  (all opaque). The mandatory re-emit cannot be discharged, so editing would
  strand a stale offset → refuse. (A last-function edit needs no re-emit and is
  allowed even with no `.gl`.)
- `OpaqueFunctionBody` — the function has opaque bytes *between* typed tokens, so it
  is not token-addressable. (Captured functions in this class are fully typed
  except a trailing module tail, so this does not bite them.)
- `NoSuchFunction` / `TokenRange` / `NotALiteral` / `ValueNotNarrowable` — ordinary
  argument guards.

## The differential gate — edits verified byte-exact against the oracle

`crates/c2-harness/tests/edit_differential.rs` (toolchain-gated, like
`differential.rs`) reproduces each P0.6a experiment as a first-class **edit**:
capture a bundle → `IlModel` edit → write → replay through standalone c2
(`Toolchain::replay_within`, TIMEOUT-bounded) → assert byte-exact to a native
capture of the equivalent source, both replayed to a **fixed `-Fo`** so the
embedded `S_OBJNAME` cannot confound the compare.

| Test | Edit | Verified byte-exact to |
|------|------|------------------------|
| **A** | widen the only literal `a+5` (+4 B), single fn | the unedited baseline (semantic no-op) |
| **D** | widen fn0 of the 3-fn `mvp_lit` (+4 B), `.gl` re-emitted | the unedited baseline (non-last, offset re-emit exercised) |
| **C** | same `.ex` as D but a STALE `.gl` (re-emit skipped) | **not** the baseline (crash/wrong obj) — proves the re-emit is load-bearing |
| **E** | insert `+5` term: `a+5` → `(a+5)+5` (+6 B) | a direct `a+5+5` capture (c2 re-folds → `addi …,10`) |
| **F** | delete `+c` term: `a+b+c` → `a+b` (−7 B) | a direct `a+b` capture (`c` left unreferenced) |

Portable unit tests (`codec.rs`, no toolchain) cover the mechanics on hand-built
bundles: the `GlOffset` vector bumps by exactly the delta for a non-last edit and
is unchanged for a last-fn edit, `encode(edited)` has the claimed length, the
edited bundle re-parses, and every out-of-scope edit is refused.

## Scope line and what is NOT here

- **In K3a:** statement-grain length edits within one function's body — varint
  literal widen/narrow, operand-stream term insert/delete.
- **OUT (K3b):** whole-function add/remove. Needs `.gl` per-function *record*
  framing (name + offset record) and the `.sy` record removed together; the `.gl`
  record boundary is still one of K1's opaque spans. Refused fail-closed today.
- **Not blocked by K2b** (the `.ex` header / FnHeader interior): both are
  length-*invariant*, so a length rewrite never touches them.

## What T-A search can now execute (verified)

Per-function `.ex` extraction puts every candidate in the "edit the last/only
function" regime, so the common length-changing move needs **no `.gl` surgery** and
is verified end-to-end: literal widen/narrow (A/B), term insert (E), term delete
(F). Non-last edits are equally legal and now mechanically discharge the offset
re-emit (D). Whole-function structural moves wait on K3b.
