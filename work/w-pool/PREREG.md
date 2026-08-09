# w-pool — PREREGISTRATION

    Lane:      w-pool  ·  board rows #2560–#2589  ·  rung docs/rungs/2026-08-09-w-pool.md
    Branch:    w-pool, worktree off master **7309a02f** (the `merge w-vec` commit)
    Commission: convert `src/system/utl/Pool.cpp` and/or `src/system/utl/EncryptXTEA.cpp`,
                TU match 20 -> 21 (or 22). Take the smaller TU first; price both.
    Frozen:    BEFORE the first `crates/` change and BEFORE the first fixture line.
               `git status` is empty of tracked modifications at this commit;
               everything measured so far lives in `work/w-pool/` and was produced
               by the BASE binary `work/w-pool/c2rs-base` (md5 `f3db5bf7b507193284ba3f1bc12d1603`,
               copied out of `target/release/` before any edit — board #2409).

    Workload stamp: dc3 `d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0`, tracked tree
               CLEAN (`git diff --quiet HEAD`). Toolchain
               `compilers/X360/16.00.11886.00`, wibo `1.0.1-23-g4a9dd6f`, known-good
               `1.0.1-23`, not stale. Flags = the workload's own
               `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`.
    Base:      878-TU scan `match 20 · mismatch 0 · codegen-gap 0 · vocab-gap 851 ·
               port-error 0 · capture-fail 7`; workspace tests **1,422 passed / 0
               failed / 39 targets**; `#[test]` **1,432**; 29 files under
               `crates/*/tests/`; **334** fixtures.

---

## 0. WHAT IS ALREADY MEASURED — findings, not predictions

Recorded here so the sections below cannot be read as retrodiction. Every line is
the base binary's own output on this tree, before any change.

### 0.1 THE BINDING PREDICATE, CHECKED FIRST (CEILING §11.4 item 8)

`work/w-pool/three_base.jsonl`, three rows, `gate_cause` / `gate_causes` from
`w-vec`'s new instrument:

| TU | `.ex` B | `fn_names` | `fn_total` | `fn_in_class` | first cause | ALL causes |
|---|---:|---:|---:|---:|---|---|
| `Biquad.cpp` | 3,694 | 1 | 2 | 0 | `body-out-of-class` | `body-out-of-class` |
| `EncryptXTEA.cpp` | 4,547 | 4 | 5 | 1 | `body-out-of-class` | `body-out-of-class` |
| `Pool.cpp` | 3,589 | 2 | 3 | 0 | `body-out-of-class` | `body-out-of-class` |

**`fn_names < fn_total` on all three, and it is NOT `mmio.cpp`'s trap.**
`fn_names` is `c2_il::mangled_names(gl).len()`, whose acceptance requires
`bytes[1].is_ascii_alphabetic()` and so drops every `??`-prefixed name — both
TUs have a constructor. The **gate** uses `gl_defined_names` =
`gl_defined_names_framed(gl, sep26=true, codec::gl_offset_framed)`, and
`Bindings::per_record` applies exactly the two checks `decode_causes` publishes as
`bind-record-count-ne-segments` and `bind-offset-ne-segment-start`. **Neither
fires on any of the three, and no `gl-stop-*` fires either — so
`Bindings::per_record` returns `Some` and binds every record 1:1 to its segment,
in order, on all three TUs.** This is the first frontier TU pair this week whose
binding predicate passes.

### 0.2 …AND THE CAUSE SET UNDERSTATES, BY CONSTRUCTION

`decode_causes` evaluates `shape-token-unresolved`, `label-stride-mismatch`,
`label-counter-unreadable` and `locally-defined-callee` over **the functions that
DID build** — 0 of 3 on `Pool.cpp`, 1 of 5 on `EncryptXTEA.cpp`. Those four gates
are therefore **vacuous** on these TUs and can fire the moment the reader admits a
body. `unclaimed-gl-symbol` is the exception and is *strengthened* by an empty
function list (`accounted` = the bound names only), so its pass is real.

### 0.3 THE REFERENCE OBJS, CAPTURED BY THIS LANE

`work/w-pool/ref/{Pool,EncryptXTEA}.obj`, `scripts/gt_dump.py`.

| | `Pool.obj` | `EncryptXTEA.obj` |
|---|---:|---:|
| size | 1,199 B | 1,942 B |
| sections / distinct names | 7 / **4** | 10 / **5** |
| symbols | 20 | 34 |
| **relocations, whole file** | **0** | **5** |
| `.pdata` | **no** | yes (+1 `ADDR32`) |
| `$M` / `$T` label symbols | **0** | 3 |
| `__savegprlr_N` / `__restgprlr_N` | **0** | 2 |
| minted intrinsic external | **0** | `memcpy` |
| `_fltused` / `__real@` | **0 / 0** | 0 / 0 |
| `.text` COMDATs | 3 (80 + 28 + 24 B) | 5 (16 + 12 + 32 + 116 + 96 B) |
| `bytefrac` exact / denominator | **0 / 132** | 16 / 272 |

### 0.4 THE LADDERS, RE-DERIVED FROM AN EMPTY SINK (`work/w-pool/ladder.sh`)

| TU | rungs | terminal | still standing at the terminal |
|---|---:|---|---|
| `Pool.cpp` | **8** | `expr-jump` | `expr-call-in-expr-other` |
| `EncryptXTEA.cpp` | **8** | `expr-jump` | `expr-convert-target-A882`, 2 bodies at sink-poison |
| `Biquad.cpp` (not this lane's) | **0** | `expr-cmp-eq` | `expr-call-in-expr-recv-load-then-plumbing-0x3A` |

### 0.5 THE CELL WALK — the head of `Pool.cpp`'s chain, cut to a 2-statement body

`work/w-pool/probe/*.cpp`, base binary, `c2rs census` + `c2rs diff`:

```text
  p4  p->mFree = (char*)v;                       store-leaf   Port=Match  BYTE-EXACT
  p9  *(void**)v = p->mFree;                     store-leaf   Port=Match  BYTE-EXACT
  p5  *v = p->mFree;            (typed)          store-leaf   Port=Match  BYTE-EXACT
  p6  q->a = (char*)v; q->b = (char*)v;          store-run    Port=Match  BYTE-EXACT
  p10 *a = 0; *b = 0;           (two bases)      store-run    Port=Match  BYTE-EXACT
  ---------------------------------------------------------------------------------
  p1  *(void**)v = p->mFree; p->mFree = (char*)v;   GAP expr-op-0x27  cflow-straight
  p7  the same two statements, REVERSED             GAP expr-op-0x27  cflow-straight
  p8  the same two statements, typed (no cast)      GAP expr-op-0x27  cflow-straight
  p11 *v = p->mFree; *v = 0;                        GAP expr-op-0x27  cflow-straight
```

**Each of `Pool::Free`'s two statements is byte-exact ALONE; their two-statement
run is out of class.** The boundary is not the cast, not the statement order, not
the base count and not control flow — all four are held fixed across a matched
pair above. It is `leaf_store::parse_store_stmt`'s **value** position: a run
admits formal- and literal-valued stores and refuses as soon as one stored value
is a member **load**, and the refusal surfaces as `expr-op-0x27`, the byte-offset
add of the loaded member's address.

---

## 1. CONVERSION CALLS — in probability form

| # | claim | p |
|---|---|--:|
| **C1** | `src/system/utl/Pool.cpp` converts; TU match 20 → 21 | **0.06** |
| **C1a** | the `Pool.cpp` DECLINE branch, with every mechanism named and sized, and a mechanism count ≥ 3 | **0.94** |
| **C2** | `src/system/utl/EncryptXTEA.cpp` converts; match → 21 or 22 | **0.02** |
| **C2a** | the `EncryptXTEA.cpp` DECLINE branch, sized at ≥ **8** independent mechanisms | **0.90** |
| **C3** | `Pool.cpp` is the cheaper of the two by every axis this lane measures (whole-obj obligations, new encoders, relocations, mechanism count) — **no axis puts `EncryptXTEA` ahead** | **0.85** |

## 2. `fnbyte-exact` DELTA — the calibrated metric (CEILING §10)

The lane expects to ship **cells and a price, no emitter arm**, so the expected
delta is the `w-fence2`/`w-vec` kind: **zero byte delta**.

| # | claim | p |
|---|---|--:|
| **C4** | `fnbyte-exact` delta is exactly **0** (35,793 → 35,793) | 0.88 |
| **C4b** | the delta is within `[-2, +2]` | 0.96 |
| **C4c** | `fnbyte-exact` does not FALL | 0.96 |
| **C4d** | per-function census and emitted census both move by **0** | 0.90 |

## 3. THE PRICE — what the two TUs owe, registered before it is written up

| # | claim | p |
|---|---|--:|
| **C5** | `Pool.obj`'s whole-obj obligation set is **EMPTY** — 0 of NC-1's seven items and 0 of NC-2's four (no `_fltused`, no `__real@`, no undefined external, no `__savegprlr_N`, no label slot, no minted intrinsic, no `.pdata`, all four section names already in `PORT_WRITER_SECTIONS`) | 0.85 |
| **C6** | **every distinct PowerPC instruction in `Pool.obj` already has an encoder** in `codegen/encode.rs` — the port owes zero new encoders on this TU | 0.85 |
| **C7** | `EncryptXTEA.obj` needs **≥ 3 new encoders** (candidates: `rldicl`/`clrldi`, `rldimi`, `stdx`, `stdu`, the record form of `addic`) | 0.85 |
| **C8** | `EncryptXTEA.cpp` carries **≥ 4 NC-1/NC-2 whole-obj obligations** that `Pool.cpp` carries **zero** of | 0.88 |
| **C9** | the band-2 `bclr` fold (`Pool::Alloc`, `Pool::Free`) is refused **by name** somewhere in `crates/`, i.e. it is a declared out-of-class shape rather than an unrecorded gap | 0.75 |

## 4. THE INHERITED PRICES THIS LANE REPLACES

| # | claim | p |
|---|---|--:|
| **C10** | **neither** ladder terminates at `expr-chain-noform-0xBD` — refuting `w-xlr` §10's `LIFTED→LIMIT` band bound for both TUs | 0.90 |
| **C11** | `EncryptXTEA.cpp`'s re-derived reader-ladder length is **< 26** (the inherited `w-xlr` figure) | 0.90 |
| **C12** | `Pool.cpp`'s re-derived reader-ladder length is **≠ 7** (`w-conv`'s inherited figure) | 0.80 |
| **C13** | `w-subclass`'s *"1 blocked fn with NO CFG class"* row for `Pool.cpp` is **stale** — all three bodies now carry a CFG class | 0.80 |

## 5. CELLS — what ships, and what each must show

Three fixtures. **Ordered so every fence is live**, and each `_neg` is proven live
by a must-fail mutation recorded in the rung (six of the last ten lanes shipped an
inert or confounded `_neg`).

| cell | one thing changed vs its control | registered verdict at `/O1` AND `/Ox` | p |
|---|---|---|--:|
| **A** `wpool_store_leaf_member_value.cpp` | — (the POSITIVE control) | **match**, byte-exact, at both modes | 0.80 |
| **B** `wpool_store_run_member_value_neg.cpp` | A, plus ONE more store | `codegen-gap` or `vocab-gap`, **never** `mismatch` | 0.90 |
| **C** `wpool_guard_bclr_fold_neg.cpp` | B, plus a null guard whose arm is a bare return | `codegen-gap` or `vocab-gap`, **never** `mismatch` | 0.90 |

* **C14** — cell **A** grades `Port=Match` byte-exact at `/O1` **and** `/Ox`: p **0.80**.
* **C15** — cells **B** and **C** carry **distinct** first blockers, so a single
  refusal fixture would have made `Pool.cpp` look one repair from a match: p **0.45**.
* **C16** — no cell needs a `// c2rs-profile:` marker; all three compile at the
  default `/Ox /GS- /c` (#2330–#2335): p **0.85**.

## 6. TEST / TARGET DELTA — registered as a number, per #2510

`w-vec` §9.2's transferable finding is that taking the previous lane's *published
number* literally beat every attempt to re-derive one, and that the half which
missed was the half nobody re-derived. So both halves are registered explicitly.

| # | claim | p |
|---|---|--:|
| **C17** | `#[test]` DELTA is **+4** (1,432 → 1,436); `±3` is the whole claim | 0.60 |
| **C18** | cargo **targets 39 → 40** — this lane adds exactly one new integration-test *file*, and a new test file is a new target | 0.75 |
| **C19** | tests **passed** rises by the same +4 with **0 failed** at both ends | 0.85 |

## 7. NEUTRALITY, GATE, AND THE PRE-ARMED FAILURES

| # | claim | p |
|---|---|--:|
| **C20** | **878 TUs BY NAME**: 0 only-in-base, 0 only-in-tip, **0 CHANGED**, and the direction of every moved verdict reported (expected: none) | 0.90 |
| **C21** | every `gap-metric` key accounted: **257 keys at both ends, `diff` empty** | 0.80 |
| **C22** | all **334 + N** fixtures at `/O1` AND `/Ox`, under BOTH binaries, list regenerated after the last fixture and `wc -l`-checked: **0 changed by name** apart from this lane's own new cells | 0.85 |
| **C23** | `mismatch` is **0** at all three levels and in all 18 gate lanes, the expr sweep and the mode cross | 0.97 |
| **C24** | full gate **18/18 PASS**, 0 FAIL / 0 SKIP / 0 NO-RESULT | 0.85 |
| **C25** | `c2rs selftest` green — 334 + N PASS, **0 ERROR** | 0.90 |
| **C26** | `scripts/board_audit.sh` all five zero; `rung_registry` 2 passed with `INDEX.md` regenerated by `scripts/gen_rung_index.sh` | 0.90 |
| **C27** | `hatch-red` **REFUSES on a PRE-EXISTING failure**, reproduced at master with this lane's `crates/` and `fixtures/` reverted. Registered kind: **`HATCH-STALE`** (#2511's, the freshest observation), with `HATCH-DRIFT` (#1406) the alternative | 0.80 |

### 7.1 THE UNNAMED-REFUSAL BUDGET — ONE

Budgeted: **1**. Pre-armed places, in the order they are expected to bite:

1. **`git checkout <rev> -- path` STAGES** (#2512). Every counterfactual in this
   lane is a temporary commit + `git reset --hard`, and every commit carries an
   explicit `-- <pathspec>`. The base binary is already copied out (#2409) and
   will not be rebuilt from a `git checkout master -- crates/`.
2. **A new integration-test file is a new cargo target** (#2510). Registered
   explicitly at C18 rather than assumed to be neutral.
3. **`_neg` inertness.** Cell B's fence must be `leaf_store`'s value clause and
   not some earlier gate; cell C's must be the guard and not cell B's clause
   again. Each is proven by a must-fail mutation, not by the verdict alone.
4. **The reported key's LAYER** (#1416, CEILING §11.4 item 5). `expr-op-0x27` is
   confirmed against a **2-statement body with no control flow** (`p11`), which is
   the same discipline `w-nc` used on `expr-jump`; the ladders' terminal
   `expr-jump` is NOT so confirmed and is reported as a ladder terminal only.
5. **Peer sessions advancing master.** `git log` re-checked before staging, every
   non-authored commit audited, rebase before reporting.

### 7.2 WHAT THIS LANE WILL NOT DO

* It will **not** widen `Bindings::per_record`. It already binds these three TUs.
* It will **not** add a name to `PORT_WRITER_SECTIONS`: factor C is already true
  on both TUs (4 of 10 and 5 of 10 names) and a name with no caller inflates C and
  converts nothing (#278, #301).
* It will **not** add a `gap-metric` key, so C21 can be a `diff` rather than an
  enumeration.
* It will **not** use the counterfactual form of the label-counter measurement
  (`wb-label` #2430–#2440). `Pool.obj` has **zero** `$M`/`$T` symbols, so the
  channel is registered at **0 slots** and `LABEL_COUNTER.md` §7.6 is not needed.
* It will **not** ship an emitter arm fitted to n = 1 for a body it has one
  witness of (`w-blockir` #2306: *two witnesses are not a rule*).
