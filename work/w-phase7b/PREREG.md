# w-phase7b — PREREG

Frozen **before the first `crates/` change, the first probe cell and the first
fixture line**. Scored in `docs/rungs/2026-08-10-w-phase7b.md` §9.

    Lane:      w-phase7b, worktree branch `worktree-agent-a118156b5f2833f6a`
               off master `a0354210` (the w-decouple merge).
    Workload:  dc3 `a8cb9ca639df2e938553ae24200307fa7a31abce`, `src/` CLEAN
               (two dirty tracked files, `config/373307D9/symbols.txt` and
               `scripts/target_symbol_map.json`, neither under `src/`).
               Toolchain `compilers/X360/16.00.11886.00`, wibo at the sibling
               build. The committed workload list and flags, USED AS COMMITTED
               and never regenerated (#2700).
    Base:      `work/w-phase7b/c2rs-base`, md5 `a39a8bfc6796d11beb14d252c49dbffe`,
               copied out of `target/release/` **before the first edit** (#2409).
    Target:    `src/system/decomp_pch.cpp` and `src/system/math/vec.cpp` — the
               `projection-divergence` pair, `CEILING.md` §2.2/§2.5/§11.3.

---

## 0. What is already measured at the base, BEFORE any change

Read out of this lane's own base scan (878 TUs, `work/w-phase7b/base_scan.jsonl`)
and its own captures. Recorded here as findings, not predictions, so the score
below cannot claim credit for them.

```text
match 23 · mismatch 0 · codegen-gap 0 · vocab-gap 848 · port-error 0
capture-fail 7 · frontier 4 · fnbyte-exact 35810 · fnbyte-differs 1898
fnbyte-partial 10 · fnbyte-refused 114622 · 262 gap-metric keys
emit-predicate-worth 124 · frontier-if-a 126 · factor-a 28 · factor-d 23
```

* **§11.4 item 8, answered with `gate_cause` and nothing else.** Both TUs read
  `gate_cause = gl-stop-26-introduced`, `gate_causes =
  [gl-stop-26-introduced, body-out-of-class]`. Neither binds.
* `decomp_pch.cpp`: `.ex` 299,107 B / **1,312** segments, `fn_names` 316,
  `fn_in_class` 383. Reference obj **901 B, 5 sections, 14 symbols, ZERO
  relocations, ZERO `.text`** — shell + one 4-byte `.rdata` COMDAT (`sel=2`,
  `ff ff ff ff`) carrying one external, `?npos@?$basic_string@…`.
* `vec.cpp`: `.ex` 170,578 B / **811** segments, `fn_names` 150, `fn_in_class`
  240. Reference obj **1,791 B**, `.rdata` COMDAT + two `.text` COMDATs +
  non-COMDAT `.data` + `.bss`.
* **`body-out-of-class` fires on 848 of 848 refused TUs** at this base, so no
  `.gl`/binding repair alone crosses the gate anywhere in the workload.
* **The `.gl` record framing is the new fact and it is a BASE measurement.**
  `codec::gl_offset_framed` pins `gl[o-5] == 0x10`, which constrains the
  *preceding* field's value to `[0x1000, 0x10FF]`. Under it the walk frames
  **31 of 1,312** records on `decomp_pch.cpp` and **36 of 811** on `vec.cpp`;
  relaxing that one byte to "PREV < 0x10000" frames **612** and **369**. The
  control — `EncryptXTEA.cpp`, a matching TU — frames **5 of 5** under both.
  `work/w-phase7b/frame.py`.

## 1. The registered claims

| # | claim | p |
|---|---|--:|
| **C1** | `src/system/decomp_pch.cpp` converts — TU match 23 → 24 | **0.12** |
| **C1b** | `src/system/math/vec.cpp` converts — match 23 → 24/25 | **0.03** |
| **C1c** | *the decline branch*: if neither converts, every mechanism is named and **script-counted per TU**, and at least one of them is a mechanism **not in `w-vec` §4's five** | **0.85** |
| **C2** | `fnbyte-exact` delta is **exactly 0** (35,810 → 35,810) | **0.88** |
| **C2b** | delta in `[−2, +2]` | 0.96 |
| **C2c** | `fnbyte-exact` does **not fall** | 0.94 |
| **C3** | **THE DECIDING ROW.** The blocker on `decomp_pch.cpp` is neither codegen nor a section name nor `_fltused`: it is that **the port has no licence to say the function emit set is empty**, and — the antecedent the claim actually needs — **that licence cannot be derived from the IL the port reads, because `.gl` carries a body-start offset for only a fraction of the `.ex` segments** (measured: 612 of 1,312 under the widest sane framing, and the missing offsets are **absent from the `.gl` byte-for-byte**, not merely unframed) | **0.80** |
| **C4** | `emit-predicate-worth` stays **124** and `frontier-if-a` stays **126** at the tip | 0.92 |
| **C5** | **`CEILING.md` §2.5's *"Both need factor A alone"* is REFUTED at the byte level** — a perfect emit-set model, dropped into today's `PortC2::build`, converts **neither**, because the emit set has to be applied in front of a gate whose binding the `.gl` cannot satisfy | 0.85 |
| **C6** | no production emitter can emit a **COMDAT `.rdata`** data group; `emit_data_obj` excludes COMDAT by an explicit clause | 0.88 |
| **C7** | mismatch stays **0** at all four levels — 878 TUs, 334+ fixtures × `/O1` **and** `/Ox` × two binaries, 18 gate lanes, the sweep and the cross | 0.96 |
| **C8** | 878-TU verdict neutrality **BY NAME**: 0 changed, 0 toward acceptance, 0 away | 0.90 |
| **C9** | `#[test]` DELTA **+4**, `±3` is the whole claim; cargo targets **39 → 39** if no new integration-test *file* lands, **39 → 40** if one does — registered as **39 → 40** | 0.55 |
| **C10** | ≥ 1 unnamed refusal fires at a pre-armed place (§2) | 0.45 |
| **C11** | the `gap-metric` key **count** stays 262 and no key changes value | 0.80 |
| **C12** | `hatch-red` REFUSES, and the refusal reproduces at the merge-base in a detached tree — i.e. it is **pre-existing** | 0.85 |
| **C13** | T1 ALL-EXACT-NO-MATCH at the tip is still **1** (`vec.cpp`) and T1b still **1** (`decomp_pch.cpp`) | 0.90 |

**C3 is the row that decides the lane and it is the row that can go wrong.**
Its antecedent is *"the offsets are absent from `.gl`"*, and the claim needs
exactly that — not *"the reader does not frame them"*. Those are different
statements and only the first forecloses a reader repair. It is checked in the
rung by searching the raw `.gl` for the literal `80 <LE32 offset>` of each
unframed `.ex` split point; **if that search finds them, C3 is VOID and the
lane's decline is worth much less**, because a record walk would then be a
reader rung rather than an impossibility.

**Unlosable rows, flagged as such:** C2c, C7, C8 and C11 cannot be lost by a
lane that ships no emitter arm; they are registered so that a lane that *does*
ship one has them to lose. C13 is unlosable if C1/C1b both miss.

## 2. Pre-armed places for an unnamed refusal

1. A COMDAT `.rdata` writer arm that resolves every symbol and gets the aux
   `CheckSum` or the `Selection` byte wrong — invisible to any per-function test.
2. A widened `gl_offset_framed`: the walk is also `bind::defined_name_set` and
   `gl::plain_external_defined_names`, so a widening of the walk is a
   **tightening** of the inline fence (#2622/#2623) — the shape that cost
   `w-front5` −1 `fnbyte-exact`.
3. A new whole-TU arm placed in front of `functions()` (where `dyninit_tu` sits)
   that fires on a TU it was not measured on.
4. `git checkout <rev> -- path` **stages** (#2512); `hatch_red.py` discards
   uncommitted `crates/` edits while printing "final crates/ diff: EMPTY"
   (#2668). Commit before every gate row and every mutation script.
5. A `_neg` fixture with more than one cell, where an over-fenced cell grades
   none of its clauses (#2698/#2699).

## 3. What this lane will NOT do

* It will **not** widen `Bindings::per_record` to bind fewer names than
  segments. That is the one change that lets a wrong obj out of the gate on 848
  TUs.
* It will **not** ship an emit-set arm whose licence is *"no `.gl` record says
  otherwise"*. On a TU where `.gl` describes 612 of 1,312 bodies that is
  `docs/STATUS.md` trap 5 — absence reads as success — with a wrong obj at the
  end of it.
* It will **not** add a name to `PORT_WRITER_SECTIONS`. Both TUs are already
  inside factor C.
