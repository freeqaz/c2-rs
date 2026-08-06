# w-refbind — PREREGISTRATION

    Lane:    w-refbind, branch `w-refbind`, worktree off master `ec535b0`
    Board:   #839 (the reference-binding bisection), #837 (H-self)
    Rows:    this lane may use ONLY #856–#865
    Ships:   nothing under `crates/` unless §4's bar is met, which it is not
             expected to be

This file is committed **before any probe of this lane exists**. Every grid this
lane runs is declared here first; a grid added later gets a **dated addendum**
below §5, written and committed before the generator runs.

---

## 0. Provenance, stated so it cannot be read as a prediction

`work/w-refbind/baseline_scan.txt` was collected **before** this file was
written — it is the baseline, not a probe of any hypothesis here. Its digits are
recorded in §0.1 as provenance and are **UNSCORED**. The prediction about the
scan is R7, which is about the run at the *other* end.

### 0.1 The baseline, at `ec535b0`

```text
  match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · capture-fail 7
  A 28 (LO 27) · B 338 · C 169 · D 10 · E 2
  B∧C 151 · A∧B∧C 27 · A∧B∧C∧D 8 · FRONTIER 17 · frontier-if-A 139
  FBM 0.16654 · fnbyte-exact 29802 · fnbyte-differs 0 · fnbyte-partial 9375
  62 `gap-metric` lines
```

Every digit reproduces `docs/rungs/2026-08-06-w-alloc2.md` §1.

## 0.2 What is already on the record, and what this lane owes

Board #839 says the C++ reference binding `L& q = s->inner;` *"moves BOTH the
schedule and the allocation"*. Reading the two prior lanes' committed grid
outputs — `work/w-alloc2/bisect.out`, `opgrid.out`, `freshgrid.out` — the
evidence behind that sentence is:

| producer | reg uses | const uses | binding | winner | source |
|---|---:|---:|---|---|---|
| `add  rX,4,5` (`u+v`)   | 1 | 1 | no  | const | opgrid `G-add-1v1` |
| `add  rX,4,5`           | 1 | 1 | yes | const | freshgrid `F4-add-r1k1` |
| `add  rX,4,5`           | 2 | 1 | no  | prod  | opgrid `H-add-2v1` |
| `add  rX,4,5`           | 2 | 1 | yes | prod  | freshgrid `F4-add-r2k1` |
| `slwi rX,4,3` (`u<<3`)  | 1 | 1 | no  | const | opgrid `G-shift-1v1` |
| `slwi rX,4,3`           | 1 | 1 | yes | const | freshgrid `F4-shift-r1k1` |
| **`slwi rX,4,3`**       | **2** | **1** | **no**  | **prod**  | opgrid `H-shift-2v1`, bisect `B2`…`B5` |
| **`slwi rX,4,3`**       | **2** | **1** | **yes** | **const** | freshgrid `F4-shift-r2k1`, bisect `B0`/`B1` |

**The allocation half of #839 rests on exactly ONE (spelling, use-count) point:
the shift at 2 uses against a 1-use constant.** Every other pair that varies the
binding with everything else fixed agrees. The schedule half is broader — the
binding moves the emitted producer order in *both* of `bisect.out`'s ladders,
including `C0`/`C1` where the winner does **not** change. That asymmetry is the
thing this lane sets out to price, and R2 registers a number that can lose.

---

## 1. R1 — the SCHEDULE moves, pairwise and in one direction

> **Registered:** in every gradeable pair of cells that are identical but for the
> binding, the binding moves the **constant** producer EARLIER in the emitted
> instruction order. Stated as a pair predicate so it is not satisfiable by an
> average.

**LOSES** if any pair shows the order unchanged, or the constant moving later.
Scored over `bindgrid.py` (§3), ≥ 24 gradeable pairs. Partial outcomes are
reported as a fraction with both counts printed, never as a status.

## 2. R2 — the ALLOCATION move is confined to the shift, and to ONE threshold

Define `T(spelling, binding)` = the smallest register-producer use count at which
that producer takes `r11` against a **1-use** constant, over `1..5`.

> **Registered, eight digits:**
>
> | spelling | `T(·, no binding)` | `T(·, binding)` |
> |---|---:|---:|
> | `self` — `(int)&s->inner` stored into `s->inner` | **1** | **1** |
> | `addi` — `u + 5` | **2** | **2** |
> | `add` — `u + v` | **2** | **2** |
> | `shift` — `u << 3` | **2** | **3** |

**LOSES** on any digit. In particular it loses if `add` or `addi` shifts its
threshold under the binding — which is the direction that would make #839 a
general allocation effect rather than a one-spelling one — and it loses if the
shift's threshold under the binding is 4, 5, or never reached.

`T = ∞` (never reached in `1..5`) is a distinct recorded outcome, not a miss
folded into 5.

## 3. R3 — the bisection: it is the ADDRESSING, not the declaration

The binding as spelled in every prior grid bundles four changes at once. §3's
grid varies them one at a time.

> **Registered:** a cell that **declares** `L& q = s->inner;` but spells its
> stores `s->inner.aN` (mode `ref-unused`) behaves like **no binding** — same
> emitted order and same registers as mode `none`. A cell that binds a pointer
> (`L* q = &s->inner;`, stores `q->aN`, mode `ptr`) behaves like the **reference**
> — mode `ref`.

**LOSES** if the bare declaration alone reproduces the `ref` behaviour, or if the
pointer spelling behaves like `none`. Either loss is more interesting than the
hit: the first would make it a symbol-table effect, the second would make it
reference-specific rather than an aliasing/addressing effect.

Modes graded: `none`, `ref`, `ref-unused`, `ptr`, `ptr-unused`, `ref-other`
(a reference bound to a *different* sub-object, stores spelled directly),
`local-int` (an unrelated named local).

## 4. R4 — the IL differs, and the bound spelling's `.ex` is LARGER

c2's only input is the IL, so two spellings that emit different bytes **must**
have different IL; that half is a tautology and is not what is registered.

> **Registered:** for the deciding pair (`shift`, reg 2 uses, const 1 use,
> `ref` vs `none`), the captured `.ex` stream of the **bound** spelling is
> **strictly larger** than the unbound one's.

**LOSES** if the two `.ex` streams are byte-identical (which would put the whole
effect in `.sy`/`.li` — a *symbol-table* dependence in c2's scheduler, and by far
the most valuable outcome available to this lane), or if the bound one is
smaller or equal.

## 5. R5 — H-self is REFUTED on a binding-varying frozen holdout

`H-self` (#837): rank producers by `KEY(p) = 2·uses(p) + (3 if p's value is
stored into the object it points at else 0)`, descending; pool `r11`, `r10`, …

> **Registered:** over a frozen holdout of ≥ 30 graded cells whose axes are
> **the binding mode and the consumption pattern** — neither of which
> `opgrid`/`selfgrid` varied — H-self records **≥ 3 misses**.

**LOSES** if it records 0, 1 or 2. A win here is a refutation of H-self and this
lane **STOPS** at it: the refutation is the deliverable and H-self is not to be
patched into a new key on the cells that killed it.

**Freezing discipline (w-magic's).** The generator writes every cell's source
**and its H-self prediction** to `work/w-refbind/holdout/` and
`work/w-refbind/holdout_pred.tsv`, and that file is **committed before a single
cell is compiled**. The grader reads the frozen predictions; it does not
recompute them. The commit SHA of the freeze is quoted in the rung.

No cell of `opgrid.py`, `selfgrid.py`, `freshgrid.py`, `allocgrid.py`,
`gapgrid.py` or `bisect.py` is re-scored in the holdout.

## 6. R6 — the anchor replay, which can lose

> **Registered:** re-compiled in this worktree, `opgrid H-shift-2v1` reproduces
> **prod `r11`, const `r10`** and `freshgrid F4-shift-r2k1` reproduces **const
> `r11`, prod `r10`**.

**LOSES** if either differs. That would mean the record, the toolchain or the
flags moved under the two prior lanes, and every number quoted from them —
including this file's §0.2 table — would be void.

## 7. R7 — nothing ships, and the warranty does not move

> **Registered:** all **62** `gap-metric` lines are byte-identical at both ends,
> checked by `diff` and not by reading a summary; `fnbyte-differs` is **0** at
> both ends; TU match is **10** at both ends; `scripts/gate.sh --jobs 6` is
> 18/18 PASS with 0 mismatch; `cargo test --workspace --release` reports the same
> **27 targets** at both ends and `909 passed / 0 failed` unless this lane adds a
> test, in which case the delta is stated and measured at both ends rather than
> inferred by subtraction.

**LOSES** on any of those.

---

## 8. The grid, declared before it exists

`work/w-refbind/bindgrid.py` — one `.cpp` per cell, compiled through
`work/w-frame/refobj.sh` at the **workload's own flags** (`work/dc3-workload/flags.txt`,
read from the file, never transcribed), disassembled with `scripts/gt_dump.py`,
graded against real `c2.dll` bytes. Ships nothing.

* **producer spellings** — `self` (`(int)&s->inner`), `addi` (`u+5`),
  `add` (`u+v`), `shift` (`u<<3`)
* **binding modes** — the seven of §3
* **register-producer use counts** — 1..5
* **constant use counts** — 1 (for R2's thresholds), plus 2 and 3 on a subset
* the constant is always `li rX,7` and always stores **first** in source, as in
  every prior grid, so this axis stays comparable

### 8.1 The two instrument defects #843 records are enforced here, not assumed

1. **Extended mnemonics.** `u << 3` prints as `slwi`, not `rlwinm`. Every
   producer regex is written against what `gt_dump.py` actually prints, and any
   cell whose producer regex matches **zero** or **more than one** distinct
   register is counted `out-of-regime` and printed as such.
2. **#644 — a producer is not one instruction.** The register a producer is read
   out of must be **written exactly once** in the body, or the cell is
   `out-of-regime`. No positional or offset-based reader is used anywhere.

`reached`, `graded`, `hit`, `miss` and `out-of-regime` are five separate printed
counters. An ungraded cell is never scored as a pass (STATUS trap 5).

### 8.2 What a green anything does NOT buy

Board **#841**: a rule measurably wrong on 20 of 81 cells left all 62
`gap-metric` lines byte-identical, because no register-derived producer can reach
`alloc::allocate` from today's emitter (#840). **This lane's gate and scan
therefore say nothing whatever about any allocation or schedule rule it
measures**, and R7 is registered as an inertness check only.

---

## 9. Addenda

*(Each new grid gets a dated entry here, committed before its generator runs.)*

### 9.1 — 2026-08-06, before `bindgrid.py` exists: three more binding modes

§3 listed seven modes. Writing the grid made it clear that four of the seven —
`ref-unused`, `ptr-unused`, `ref-other`, `local-int` — are all the *same*
control: a declaration the stores do not use, which `/O1` may delete outright
before c2 ever sees it. If they all behave like `none` that is a **trivial**
confirmation of R3, not the informative one.

So three modes are **added**, each of which forces a *named temporary that the
stores actually address through*, which is the thing the reference spelling
plausibly does to the IL:

* **`iptr`** — `int* p = (int*)&s->inner;` and stores `p[N]`. Same addresses, a
  different pointee type, still a named address temp.
* **`outer-ref`** — `S& z = *s;` and stores `z.inner.aN` / `z.fN`. The temp names
  **r3 itself** rather than an interior address.
* **`val-temp`** — `int w = <expr>;` and stores `s->inner.aN = w;`. Names the
  **value** instead of the address, with the addresses spelled directly.

The original seven are all still graded. **R3 is not reworded** — it is scored as
written, and `iptr` / `outer-ref` / `val-temp` are reported as additional rows
that R3 did not register. If `val-temp` flips the allocation, R3's framing (*"it
is the addressing"*) is wrong in a way R3 as written cannot record, and that will
be stated as a MISS-adjacent finding rather than folded into a hit.

### 9.2 — 2026-08-06, before `refprobe.py` exists: what KIND of temp

`bindgrid.out` (committed at `8f9bc5e`) settled R3 as registered and killed
*"any named temp"* on its own added row: `outer-ref` (`S& z = *s;`, stores
`z.inner.aN`) is **none-like** while `ptr` and `iptr` are **ref-like**. The
surviving description is *"a named binding to an INTERIOR address that the
producer's stores address through"*, and `refprobe.py` bisects it further. New
registered claims, each losable:

> **R8.** The effect is about the temp being a **non-trivial address**, not about
> it being a binding. A reference bound to a sub-object at **offset 0**
> (`L& q = s->head;` where `head` is the first member) is **none-like** — same
> ORDER and same ALLOC as the direct spelling.
>
> **LOSES** if the offset-0 binding is ref-like. That loss would make the
> C++-level *spelling* the axis, with the address value irrelevant, and would be
> the more surprising outcome.

> **R9.** The effect needs the temp to be a **shared base** for two or more
> stores. Two *scalar* references (`int& x0 = s->inner.a0; int& x1 =
> s->inner.a1;`), which name the two store addresses separately and share no
> base temp, are **none-like**.
>
> **LOSES** if they are ref-like.

> **R10.** The effect is carried by the producer's **consuming stores**, not by
> the constant's. A binding used only for the constant's store, with the
> register-derived producer's stores spelled directly, is **none-like**.
>
> **LOSES** if it is ref-like.

Also graded, unregistered and reported as extra rows: an **unnamed** interior
address (`(&s->inner)->aN`), a `const`-qualified pointer binding, a binding where
only **one** of the two producer stores goes through it, a **reference formal**,
and a two-register-producer cell with no constant at all.

### 9.3 — 2026-08-06, before `ilcmp.sh` exists: R4's measurement

### 9.4 — 2026-08-06, before `holdout.py` exists: R5's population and freeze

R5 is scored on `work/w-refbind/holdout.py`, run in two phases:

* `--freeze` writes one `.cpp` per cell under `work/w-refbind/holdout/` **and**
  `work/w-refbind/holdout_pred.tsv`, which carries, per cell, H-self's key for
  both producers, its predicted winner, and the **sha256 of the source**. It
  compiles nothing.
* that file and every source are **committed**, and the SHA is quoted in the rung.
* `--grade` then compiles, **re-checks every source's sha256 against the frozen
  row**, and refuses to grade a cell whose source moved.

**Partitions**, declared here:

| | axis | cells |
|---|---|---|
| **H1** | ten producer spellings H-self has never seen — `subf`, `and`, `or`, `xor`, `neg`, `nor`, `srawi`, `srwi`, `extsh`, and a **`lwz` load** | × mode × (ru,cu) |
| **H2** | self-referential producers at **fresh addresses** — `&s->inner2` into `inner2`, `&s->inner.a4` into `inner`, and a scalar `int&` | × mode × (ru,cu) |
| **H3** | the two **non-self** controls whose H-self key ties, so the row cannot look like a confirmation | |

**mode** ∈ {`none`, `ref`} — the axis `opgrid`/`selfgrid` never varied, which is
the whole point. **(ru, cu)** ∈ {(1,1), (2,1), (1,2)}.

**The `(self, ref)` corner is SHAPED like `opgrid`'s fitted cells** (an interior
address bound to a name and stored into itself). It is graded and it is
**counted separately** from the never-fitted rows; R5's `≥ 3 misses` is scored on
the never-fitted count alone, so a miss in that corner cannot carry the claim.

R4 is scored by capturing the IL for the deciding pair
(`P1-shift-none-r2k1` / `P1-shift-ref-r2k1`) with `c2rs capture --keep-il` at the
**workload's own flags**, and comparing the five captured files byte-for-byte and
by size. The registered claim is that the bound spelling's `.ex` is **strictly
larger**; a byte-identical `.ex` is the outcome that would be worth the most and
is recorded as its own result, not as a near-miss.
