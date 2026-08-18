# PREREG — `w-deadsites`: partitioning `w-mutcensus`' GREEN population into DEAD and UNGUARDED

    Lane:   w-deadsites
    Base:   master `1744ced1`
    Frozen: this file is committed BEFORE the first probe runs. Nothing below is
            edited afterwards; every deviation is appended to
            `work/w-deadsites/deviations.md` with its reason.

Provenance: board **#3246** (`w-calleeguard` §8 F2) — *"a mutation census cannot
distinguish a DEAD site from an UNGUARDED one … `X` is the SUM OF TWO BACKLOGS
THAT NEED DIFFERENT WORK … the partition has never been taken anywhere."*

---

## 0. Reading order, stated because it affects how this prereg scores

Three things were read **before** this file was frozen, and they are declared
here so no result below can be told as a discovery:

1. **The source of every site in the population**, at `1744ced1`. Where the
   reading yields a proof of unreachability it is stated as such below
   (`CA9`, `CA10`, `CA13`, `L9`) — those four are *registered* dead, not
   discovered dead.
2. **The base 878-TU scan** (`work/w-deadsites/logs/scan_base.log`, run at
   `1744ced1` before this commit). Its **published key counts** were consulted:
   `disp-store-run-call` **8 bodies / blocked 0**, `disp-store-run-bind`
   **1 / 0**, `disp-static-scan-loop` **1 / 0**, and **no** `callee-unresolved-*`,
   `store-run-bind-*`, `call-arg-nonformal`, `call-arg-computed`,
   `call-arg-*-classified-twice` or `data-sym-*` line anywhere in the blocked-key
   histogram. That is why several census-key sites are registered UNREACHED
   *on the workload* below; the workload is **not** the whole corpus and the
   suite + gate are expected to dominate.
3. **`w-calleeguard` §4.4's reading of `bind_run_ops`**, which this lane is sent
   to upgrade from a reading to a measurement.

---

## 1. The frame — the population, and what a "site" is here

**Population = the 30 sites `w-mutcensus` §3 measured GREEN at `3835469c`.**
Four of them (`CS5`–`CS8`) were turned RED by `w-calleeguard` at `44794fa4`;
they stay in the probe set as **positive controls**, because being guarded does
not change whether an input reaches them, and `w-calleeguard` built cells that
provably do.

**So: 26 open rows + 4 controls.** Four further controls are added from
`w-mutcensus`' RED population, one per file the probe instrument touches, so a
per-file plumbing failure cannot read as "unreached".

### 1.1 Re-located at `1744ced1` — no line number is inherited

`git diff --stat 3835469c..1744ced1 -- crates/c2-il/` touches `bind.rs`,
`body/mod.rs`, `bundle.rs`, `census.rs`, `diag.rs`, `gl.rs`, `func/mod.rs`,
`lib.rs`. **`calls.rs` and `leaf_store.rs` are untouched**, which reproduces
`w-calleeguard` §5 caveat 2 exactly: 11 rows byte-identical, 15 rows re-located.
Every row below was re-found **by its site text**, never by offsetting a line
number.

| id | file (`crates/c2-il/src/func/…`) | `3835469c` | **`1744ced1`** | site text |
|---|---|---:|---:|---|
| CS2 | `census.rs` | 1242 | **1285** | `"store-run-call" => STORE_RUN_CALL_NO_CARRIER,` |
| CS3 | `census.rs` | 1245 | **1288** | `"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,` |
| CS4 | `census.rs` | 1263 | **1306** | `bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)` |
| CS5 *(ctl)* | `census.rs` | 1265 | **1308** | `"framed-call" => CALLEE_UNRESOLVED_FRAMED,` |
| CS6 *(ctl)* | `census.rs` | 1267 | **1310** | `CALLEE_UNRESOLVED_SEQ` |
| CS7 *(ctl)* | `census.rs` | 1270 | **1313** | `CALLEE_UNRESOLVED_DTOR` |
| CS8 *(ctl)* | `census.rs` | 1272 | **1315** | `_ => CALLEE_UNRESOLVED_TAIL,` |
| CS9 | `census.rs` | 1280 | **1323** | `Some(f) if opt_word_mode(opt_word).is_none() =>` |
| CA2 | `body/shapes/calls.rs` | 434 | **434** | `if slots.len() > MAX_REGISTER_FORMALS` (sym path) |
| CA6 | `body/shapes/calls.rs` | 693 | **693** | `None => return Err(refuse("call-arg-nonformal"))` |
| CA8 | `body/shapes/calls.rs` | 710 | **710** | `_ => return Err(refuse("call-arg-computed"))` |
| CA9 | `body/shapes/calls.rs` | 732 | **732** | `SlotArg::Lit(_) => …("call-arg-lit-classified-twice")` |
| CA10 | `body/shapes/calls.rs` | 736 | **736** | `SlotArg::SymAddr(_) => …("call-arg-sym-classified-twice")` |
| CA13 | `body/shapes/calls.rs` | 772 | **772** | `return Err(refuse("call-arg-source-out-of-slots"))` |
| CA16 | `body/shapes/calls.rs` | 792 | **792** | `if has_repeated_leaf(&arg_ops)` |
| CA18 | `body/shapes/calls.rs` | 803 | **803** | `if !additive_chain_canonical(&arg_ops)` |
| B2 | `bind.rs` | 929 | **974** | `if !o.comdat \|\| !o.initialized` |
| B3 | `bind.rs` | 932 | **977** | data-def `DATA_FLAG_THREAD_LOCAL != 0` |
| B4 | `bind.rs` | 939 | **984** | `init.accepted + init.residue.len() != init.records` |
| B5 | `bind.rs` | 942 | **987** | `!init.refs.get(&tok)…` |
| B6 | `bind.rs` | 946 | **991** | `bytes.len() != o.size as usize` |
| B7 | `bind.rs` | 985 | **1030** | `if o.comdat \|\| o.initialized` |
| B8 | `bind.rs` | 988 | **1033** | bss `DATA_FLAG_THREAD_LOCAL != 0` |
| BU3 | `bundle.rs` | 1940 | **1970** | `find_subslice(ex, &LO_MARKER).is_none()` — the `else { None }` arm |
| D1 | `bundle.rs` | 2423 | **2472** | `if !is_dynamic_initializer_name(&thunk_name)` |
| D2 | `bundle.rs` | 2887 | **2936** | `data_tu`'s `.in` totality |
| G2 | `gl.rs` | 2198 | **2222** | `out.retain(\|n\| !bad.contains(n))` |
| L2 | `body/shapes/leaf_store.rs` | 2257 | **2257** | `let IlOp::Load(base_tok) = b else` |
| L3 | `body/shapes/leaf_store.rs` | 2285 | **2285** | `_ => return Err(STORE_RUN_BIND_GROUP_SHAPE)` |
| L9 | `body/shapes/leaf_store.rs` | 2455 | **2456** | `if !matches!(b, IlOp::Load(_))` (2nd walk) |

Extra controls: **X1** `leaf_store.rs:2254` (`L1`, RED), **X2** `calls.rs:747`
(`CA11`, RED), **X3** `bind.rs:1036` (`B9`, RED), **X4** `census.rs:1421`
(`CS12`, RED).

### 1.2 What "the site fires" means, uniformly

Every row above is either a **raise** (`return Err(K)` / a `match` arm producing
a key) or a **gate** whose taken branch is a refusal (`return None`). The probe
is placed on **the branch the mutation removes**, so one definition covers both:
*the site fires when control reaches the point the census mutated.* For `G2` —
whose mutation is `retain` → keep-all — the site fires when `retain` actually
drops a name. For `CS4` — whose mutation drops `unwrap_or` — it fires when
`bind_key.is_some()`.

---

## 2. The method

### 2.1 Screen: one behaviour-preserving instrumented corpus run

`#3246` names the `panic!()` probe. A panic **aborts**, so the first site to fire
hides every later one, and 30 sites would be 30 corpus runs. This lane therefore
screens with a **behaviour-preserving** first-hit marker and *confirms* with
`panic!()`.

* A temporary `crates/c2-il/src/deadprobe.rs` exports
  `hit(ix: u32, id: &str)`: one `AtomicU64` bitmask, first hit per index per
  process appends `id` to the file named by `C2RS_DEADPROBE_LOG`. Later hits are
  a single relaxed atomic and cost nothing.
* One `hit(...)` call is inserted at each of the 34 sites. **Nothing else
  changes**, so the run's own pass/fail totals must be **identical to the
  baseline** — that identity is the instrument's self-check and is asserted.
* Corpus: `cargo test --workspace --release --no-fail-fast` (with
  `C2RS_REQUIRE_TOOLCHAIN=1`), `scripts/gate.sh --jobs 16 --require-graded`
  (18 lanes, the 19,556-case generated sweep, the 90,812-cell mode cross, the
  debug-profile lane), and the **878-TU workload scan**.
* **The controls are the instrument's validity check**: `CS5`–`CS8`, `X1`–`X4`
  must all appear in the log. A missing control voids the run.

### 2.2 Confirm: `panic!()`, batched

Every site the screen reports **unreached** has its body replaced by
`panic!("w-deadsites <id>")` — **all of them at once, in one patch** — and the
whole corpus is re-run. A run that completes clean confirms every one of them
unreached simultaneously; a panic names its own `id` and its site is split out
and re-run. This is `#3246`'s named method, run against the same corpus, and it
costs **one** run rather than *n*.

### 2.3 The verdict rule, written before any result

| verdict | condition | the work it implies |
|---|---|---|
| **UNGUARDED** | the site FIRES in the corpus | someone must **write a witness** |
| **DEAD** | the site does not fire **and** a proof of unreachability is stated from the source | **delete** the code — or, where the site is load-bearing for match exhaustiveness, record it as structurally dead (see §3.3) |
| **UNKNOWN** | the site does not fire and there is **no** proof | **neither.** Not dead. A statement about this corpus only |

**A site is never called dead because a probe did not reach it.** The corpus is
named in every claim: this project's suite + gate + the 878-TU workload, which
`#3254` records as a denominator 71.2 % of which never ships and
`w-corpushealth` records as one head of a tree moving 284 commits / 14 days.

---

## 3. Registered verdicts — all 30, before the first probe

Probe column = will the site FIRE in the corpus. Partition column = the verdict
the rule in §2.3 will then produce.

| id | probe | P | partition |
|---|---|---:|---|
| CS5 *(ctl)* | **FIRES** | 0.97 | — |
| CS6 *(ctl)* | **FIRES** | 0.97 | — |
| CS7 *(ctl)* | **FIRES** | 0.97 | — |
| CS8 *(ctl)* | **FIRES** | 0.97 | — |
| X1 *(ctl)* | **FIRES** | 0.95 | — |
| X2 *(ctl)* | **FIRES** | 0.95 | — |
| X3 *(ctl)* | **FIRES** | 0.90 | — |
| X4 *(ctl)* | **FIRES** | 0.90 | — |
| CS2 | quiet | 0.55 | UNKNOWN |
| CS3 | quiet | 0.60 | UNKNOWN |
| CS4 | quiet | 0.60 | UNKNOWN |
| CS9 | FIRES | 0.55 | UNGUARDED |
| CA2 | quiet | 0.60 | UNKNOWN |
| CA6 | FIRES | 0.85 | UNGUARDED |
| CA8 | FIRES | 0.85 | UNGUARDED |
| **CA9** | quiet | **0.97** | **DEAD** — proof in §3.1 |
| **CA10** | quiet | **0.97** | **DEAD** — proof in §3.1 |
| **CA13** | quiet | **0.90** | **DEAD** — proof in §3.2 |
| CA16 | FIRES | 0.80 | UNGUARDED |
| CA18 | FIRES | 0.80 | UNGUARDED |
| B2 | FIRES | 0.85 | UNGUARDED |
| B3 | quiet | 0.60 | UNKNOWN |
| B4 | FIRES | 0.50 | UNGUARDED |
| B5 | FIRES | 0.55 | UNGUARDED |
| B6 | quiet | 0.55 | UNKNOWN |
| B7 | FIRES | 0.85 | UNGUARDED |
| B8 | quiet | 0.60 | UNKNOWN |
| BU3 | FIRES | 0.60 | UNGUARDED |
| D1 | FIRES | 0.80 | UNGUARDED |
| D2 | quiet | 0.50 | UNKNOWN |
| G2 | FIRES | 0.50 | UNGUARDED |
| L2 | quiet | 0.70 | UNKNOWN |
| L3 | quiet | 0.65 | UNKNOWN |
| **L9** | quiet | **0.97** | **DEAD** — proof in §3.4 |

### 3.1 `CA9` / `CA10` — the source declares them unreachable and this prereg agrees

`calls.rs:724` is reached only past `if syms > 0 { return … }` (`:718`) and
`if lits > 0 { return … }` (`:721`), so `syms == 0 && lits == 0`. `SlotArg::Lit`
is pushed only in the arm that does `lits += 1` and `SlotArg::SymAddr` only in
the arm that does `syms += 1`. Hence `slots` holds `Formal` only, and neither
match arm at `:732`/`:736` can be selected. The file says so itself:
*"Unreachable: `lits == 0` is exactly 'no `SlotArg::Lit` was pushed', stated
positively rather than as an `unreachable!`, because a panic in the CLI is the
failure mode this file's header records."*

### 3.2 `CA13` — dead in the shipped configuration, live under a hatch

`permutation_cycles(&arg_sources)` returns `None` only when `arg_sources` is not
a permutation of its own slots. `:747` already refused any `ix >= len` and
`:758`–`:762` already refused any repeat, so at `:771` the sources are `len`
distinct values in `0..len` — a permutation. The comment says the same and names
the condition that makes it live: `work/w-front3/hatch.py`'s
`call-arg-outer-formal` hatch.

### 3.3 Registered in advance: the DEAD half is NOT one backlog

`#3246` prices a dead site as *"a deletion"*. This prereg registers, **before
measuring**, that at least one of `CA9`/`CA10` will turn out **not deletable** —
they are arms of a `match` over `SlotArg` and removing them makes the match
non-exhaustive. Registered P = 0.90 that the "delete the code" price is wrong
for at least one row of the dead half.

### 3.4 `L9`

`w-calleeguard` §4.4's argument, re-derived here from the source at `1744ced1`:
`bind_run_ops`' first walk (`:2252`–`:2288`) consumes `ops` in threes and returns
`Err` unless slot 0 of each group is `IlOp::Load`, so a success implies
`ops.len() % 3 == 0` and every 3k-th op is a `Load`; the second walk
(`:2453`–`:2462`) re-walks the **same** immutable slice in threes. `:2456`'s
condition is therefore never true.

---

## 4. Headline registrations

| id | registration | P / interval |
|---|---|---|
| **H1** | of the **26** open rows: **DEAD 4**, **UNGUARDED 12**, **UNKNOWN 10** | 80 % interval on DEAD **[3, 7]**, on UNGUARDED **[9, 16]** |
| **H2** | `leaf_store.rs:2456` is confirmed dead — the `panic!()` probe does not fire anywhere in the corpus | 0.97 |
| **H3** | **all 8 controls fire.** A control that does not fire **voids the run** and is a campaign-stopping finding that outranks the partition | 0.93 all eight |
| **H4** | the instrumented corpus run reproduces the baseline pass/fail/target counts **exactly** | 0.90 |
| **H5** | `DATA_SYM_STRLIT_FENCED`'s **two** raise sites (`census.rs:1259`, `:1511`) are **already guarded** — `w-calleeguard` F3 called them the cheapest unguarded follow-on, and this lane registers that a mutation at each is RED before it runs one | 0.70 each; P(both RED) = 0.55 |
| **H6** | the standing fence-site census lands as **one** test in `crates/c2-harness/tests/`, keyed on **key strings and per-key counts** rather than a single integer, and is shown GREEN → RED by adding one site | 0.85 |
| **H7** | suite ends at **1,666 + k**, `2 ≤ k ≤ 8` | 0.80 |
| **H8** | 878-TU scan: **0 differing lines over all 394** prefix-anchored `gap-metric` keys, base vs tip — *including* `fnbyte-*` measured back-to-back per **#3249** | 0.90 |
| **H9** | `scripts/gate.sh --jobs 16 --require-graded` PASS at both ends; per-lane gate-count identity diff 0 rows differing with the **range length asserted** | 0.92 |
| **H10** | `git diff master..HEAD -- crates/c2-il` at the tip contains **nothing but proven-dead deletions**, each justified by its probe | 0.85 |
| **H11** | at least one row registered UNKNOWN is **not** promoted to DEAD by the end of the lane — i.e. the lane publishes a non-empty UNKNOWN bucket rather than resolving the population into two halves | 0.90 |

**H3 is the stop rule.** If any control is quiet, every colour in the run is
**void, not provisional** (`docs/rungs/README.md` probe rule 1): it is discarded,
its log kept, and the partition is not published from it.

---

## 5. Probe soundness — the checks this lane commits to in advance

1. **Control pinned by NAME, re-run in every environment.** `C1`
   (`calls.rs:431`, `syms > 1` → `syms > 2`) must take down exactly
   `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol`
   and `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone`,
   before the first probe and after the last.
2. **Every suite run carries `C2RS_REQUIRE_TOOLCHAIN=1`**, and the
   `census_gate` target's **duration** is recorded per run. Under 1 s ⇒
   `INVALID`, log kept with `.INVALID.` in its name, colour discarded.
3. **The table is DERIVED from the logs** by a tracked script, never
   accumulated.
4. **Every probe patch is verified reverted** — `git diff -- crates/c2-il`
   empty (or exactly the justified deletions) after each run.
5. **`C2RS_DEADPROBE_LOG` is a fresh empty file per run**, and the run's
   log file name records which patch produced it.

---

## 6. What this lane will NOT do

* It will **not** delete a site on the strength of a quiet probe. Only the four
  rows carrying a source-level proof (§3.1, §3.2, §3.4) are deletion candidates,
  and `CA9`/`CA10` are registered in §3.3 as probably not deletable at all.
* It will **not** re-run `w-mutcensus`' key-swap campaign. This lane measures
  **reachability**, a second colour per row, not a second campaign.
* It will **not** widen the frame past `w-mutcensus`' 63. `DATA_SYM_STRLIT_FENCED`
  is handled as a named exception (H5) because `#3246`'s sibling row asked for
  it explicitly.
* It will **not** edit `scripts/` (peer `w-coldcross`) and its permanent
  `crates/c2-il` diff is deletions only (peer `w-sizebracket`).

## 7. Board rows

`#3276`–`#3281`, allocated by the coordinator. No row is read from the
`BOARD.md` next-free pointer.
