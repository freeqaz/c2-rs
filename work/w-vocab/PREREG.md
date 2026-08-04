# W-VOCAB prereg — registered BEFORE the first `crates/` edit and before any measurement

Lane `w-vocab`, branch `wt-w-vocab`, off master `b6fa935`.
Seam: `crates/c2-il/` only.

Baseline, measured on the branch tip before any edit
(`work/w-vocab/gap-base.txt`, `capture cache: 871 hit, 7 miss`):

    match 8 · mismatch 0 · codegen-gap 0 · vocab-gap 863 · capture-fail 7
    A 28 · B 338 · C 169 · D 8 · E 2 · A∧B∧C 27 · FRONTIER 19

## 0. What this lane is for

`vocab-gap 863` is one bucket with one label (`il function decode failed`,
`gap.rs` step 3: `!IlBundle::decodes()`). Decompose it into causes with counts,
and test lane w-small's board row **AB-g** — that
`codec::gl_offset_framed`'s `gl[o-5] == 0x10` clause pins a **CodeView type
index** into `0x1000..=0x10FF`, so a TU with enough distinct types walks its
later `.gl` function records out of the framing window.

## 1. The refusal paths that exist, enumerated from the source before measuring

`IlBundle::functions()` (`crates/c2-il/src/func/bundle.rs:1121`) short-circuits
on, in order:

1. no `.gl` / no `.ex`
2. `!drectve_is_boilerplate(gl)`
3. split empty **and** an `LO` marker present
4. `Bindings::per_record(...)` → `None` (`bind.rs:445`), which is itself
   either the 1:1/offset-equality gate, or one of `gl_defined_names`'s **five
   total-refusal clauses** (`gl.rs:245`): name > 32 B away or absent, name not
   `looks_mangled`, run ending at `26`, `dllexport` linkage, `26`-introduced
   defined name
5. `bind.is_varargs(i)`
6. `parse_segment(...)` → `None` — the per-body decode
7. `shape_to_function(...)` → `None` — a CALL/data token that will not resolve
8. the label-stride gate / `gl::label_counter(gl)` unreadable
9. an unclaimed `.gl` symbol
10. a callee this TU also defines

`dyninit_tu()` is the second acceptance path; `decodes()` is their disjunction.

## 2. Registered predictions

| # | prediction | interval | how it is scored |
|---|---|---|---|
| **P1** | The **first-cause** histogram over the 863 is dominated by path 4 (`per_record` → `None`). | ≥ 700 of 863; point estimate **860** | the instrument's first-cause count |
| **P2** | Within path 4, the **type-index window** is a *contributing* cause on a majority of TUs — i.e. the wide framing (`gl[o-4]==0 && gl[o-3]==0`, `bind::emit_offset_framed`'s clause) finds strictly more records than the gate framing. | ≥ 400 of 863; point estimate **820** | count of TUs where wide-framed record count > gate-framed record count |
| **P3** | The window is almost never the **sole** cause. Making the gate framing wide converts **0** TUs from `vocab-gap`. | [0, 12], point estimate **0** | re-run `c2rs gap` with the widened framing; count the `vocab-gap` delta |
| **P4** | The pinned `80 <LE32>` field is a **monotone counter based at `0x1000`**, and adding one `struct Tn{int x;};` burner shifts a later function's field by a **constant stride of 4**. The framed→unframed flip happens exactly when the value crosses `0x10FF`, so w-small's 62/63/64 boundary is *predictable*, not incidental. | stride exactly 4; flip N predicted to ±0 | capture a burner sweep, read the field, fit |
| **P5** | I will **not** ship a framing widening, because P3 says it converts nothing. | — | see the decline clause |
| **P6** | TU match unchanged. | [8, 8] | `c2rs gap` |

**Reasoning behind P3, registered so it can be wrong in public.** Four of
`gl_defined_names`'s five total-refusal clauses are unaffected by the framing,
and one of them — the `26`-introduced defined name, board #232 — fires on any
TU that defines a COMDAT-linkage function, which is any TU that includes a
header with an `inline` or a template. Independently, the gate requires *every*
`.ex` segment to decode, and the workload's own census reads *TU distance to
match, blocked functions ≤0: 1, ≤1: 12* — so at most ~12 TUs could ever have
all their bodies in class. Both bounds are far below 863.

## 3. Decline clauses

* **D1 — the standing one.** A reproduced `Port=Mismatch` from anything this
  lane writes stops the lane and is reported immediately, not at lane end.
* **D2.** If the widened framing converts **0** `vocab-gap` TUs on the 878-TU
  workload, the widening is **not shipped**. A reader relaxation with zero
  measured effect is pure risk (board #232 is what that costs), and the
  measurement is the deliverable either way.
* **D3.** If the widened framing converts TUs *and* I cannot construct a case
  that the widening makes wrong, I still do not ship it without saying exactly
  what I tried to construct and why it did not break.
* **D4.** If the wide framing changes `per_record`'s verdict on **0** TUs, the
  AB-g hypothesis is **refuted as a cause of `vocab-gap`** and reported as such.

## 4. What this lane cannot claim

* **The coverage sweep instrument (`work/w-frame/sweep.py`) cannot see
  `crates/c2-il` at any `C2RS_SWEEP_KEEP` value** — it reads `c2-core` only
  (w-label §5, w-small §6). This lane's seam is `crates/c2-il` *exclusively*,
  so **no coverage figure will exist for any line it writes**, and silence from
  that instrument must not be read as coverage.
* "N TUs now decode" is never "N TUs now match". A TU leaving `vocab-gap`
  enters `codegen-gap`, where the port refuses it for a codegen reason.

---

## 5. P4 refined, registered mid-lane on the first 20 cells (N = 0..19 only)

Measured on `struct1_000..019`: the trailing `?f@@YAHH@Z` record's field is
**`0x1001 + 4N`** exactly, stride **4**, no exceptions. Registered *before*
capturing N ≥ 20:

* **P4a** the affine law holds to N = 90: `field(N) = 0x1001 + 4N`.
* **P4b** the gate framing (`0x1000..=0x10FF`) loses the record at **exactly
  N = 64**: `field(63) = 0x10FD` is framed, `field(64) = 0x1101` is not, and
  there is no N with a value in `0x10FE..=0x1100`. So w-small's 62/63/64
  boundary is **derivable**, not incidental — and it moves to a different N
  under a different burner stride.
* **P4c** a burner with a different type cost changes the stride and moves the
  flip to `N = floor(0xFE / stride) + 1`, checked on a second and third burner
  kind.
