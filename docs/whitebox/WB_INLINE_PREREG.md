# WB-E `wb-inline` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing in this lane
> is adopted into `crates/`.

Registered **before the first grep of `~/ghidra-projects/export/c2/`** and before
the first `cl.exe` of this lane, per board #770's standing rule (streak ~10
optimistic / 2 pessimistic / 2 hits after `wb-memcpy`). Scored in
[`WB_INLINE_FINDINGS.md`](WB_INLINE_FINDINGS.md) §9.

Lane: `wb-inline` / branch `worktree-agent-a060a9ed2e9ba8bc7`, at master
`9ed20248`.

**What was read before freezing** (so "cold" is not overclaimed): the campaign
briefs, `C2_MAP_METHOD.md`, `WB_MEMCPY_FINDINGS.md`, `WB_FRAME_FINDINGS.md` §7,
`docs/INLINE_PREDICATE.md` (all of it), `docs/DIFF_STRUCTURE.md` §2–§3,
`docs/CEILING.md` §6.1 row 2, `docs/rungs/2026-08-08-w-alloc3.md` §1–§2, and
board rows #1474/#1477/#1570. **No** disassembly and **no** obj was consulted.
`LABEL_COUNTER.md` §6.15–§6.20 was *not* read directly; the model below is
quoted as `INLINE_PREDICATE.md` §2 transcribes it.

## The incumbent this lane is trying to beat

`INLINE-P`, black-box, from `LABEL_COUNTER.md` §6.15–§6.20, graded 0.9716 on a
9,993-callee hold-out with a **2.84 % residual explicitly NOT MODELLED**:

```
index(G) =  s                                     linkage == STATIC
            s - 4*(nparams-1) - 8*[inline]        linkage == EXTERNAL
         -  48*[leaf]                             both classes

N_max(G) =  0                                     if varargs(G)
            EXTERNAL:  UNBOUNDED if index <= 64 else 0
            STATIC:    i = index/4
                       0                          i >= 65
                       UNBOUNDED                  i <= 16
                       min(9, 1 + floor(19/(i-16)))  otherwise
inline every site of G iff n_sites(G) <= N_max(G)
```

A conditional site moves the `1 → 0` ceiling from `(256,260]` to `(160,164]`.

## P1 — where the decision lives

| # | registered | direction if wrong |
|---|---|---|
| **P1.1** | The inline decision is made **inside `c2.dll`**, not by `c1xx`: the IL bundle carries an ordinary call tuple at the site and the callee's un-lowered body elsewhere in the same bundle, exactly as `wb-memcpy` §5.3 showed for the block-move tuple. | optimistic |
| **P1.2** | There is a single **cost/benefit function** — one routine that computes a number from callee properties and compares it against one or more thresholds — reachable from an inline-expansion driver, and it is findable from the flat export by the constants in the incumbent model (`0x40`=64, `0x30`=48, `0x13`=19, `0x104`=260, `0xa4`=164) rather than by a string. | optimistic |
| **P1.3** | The `/Ob` level and the favor-speed bit reach it through **globals in `.data` written by the option decoder**, in the same family as `0x10c2e310` (`WB_MEMCPY` §2.1), and at least one of them is read by the inliner. | optimistic |

## P2 — the decision function

| # | registered | direction if wrong |
|---|---|---|
| **P2.1** | **The graduated middle `min(9, 1+floor(19/(i-16)))` is not a table and not a division — it is a BUDGET LOOP.** The mechanism is "keep accepting sites while accumulated growth ≤ B", with per-extra-site cost `(i-16)` words and `B = 19`; `1 +` is the first site being free. I predict the binary contains an accumulate-and-compare, not a `__alldiv`/`idiv` against 19. | **optimistic** — this is the sharpest single claim in this PREREG |
| **P2.2** | The constant **16** subtracted inside that expression, the **64**-byte EXTERNAL ceiling and the **`i <= 16` STATIC unbounded arm are the same constant** (64 bytes = 16 words) appearing once in the binary, not three separately fitted numbers. | optimistic |
| **P2.3** | The **48-byte leaf term is a call-overhead charge**: a fixed constant added to (or, equivalently, not credited to) a callee whose body contains a call. It is a single immediate in the cost routine and it is *not* a per-call multiplier. | optimistic |
| **P2.4** | The hard caps **260** and **164** are the same quantity read at two different thresholds selected by a call-site property, and the call-site property is *whether the site is control-dependent* (`INLINE_PREDICATE.md` §2's "conditional site"). | pessimistic — I expect at most to find the two constants, not the site-side input |
| **P2.5** | **`varargs ⇒ never`** and **direct recursion ⇒ never** appear as early categorical refusals, before any size arithmetic. | optimistic |
| **P2.6** | The decision is **all-or-nothing per callee** (§6.15.1) because the count of accepted sites is accumulated on a **per-callee record**, not per call site. | optimistic |
| **P2.7** | **The 2.84 % residual will NOT be closed by this lane.** I predict the disassembly names at least one input the incumbent does not have (a candidate for the residual) but that this lane cannot obj-confirm it inside its own grid budget. | **pessimistic, and registered as the headline decline** |

## P3 — `?supershuffle@@YAXPAD@Z`

c2 inlines exactly **one** of its six callees — `?shuffle2`, 14 words — and
declines `?shuffle1`, `?shuffle3`…`?shuffle6`.

| # | registered | direction if wrong |
|---|---|---|
| **P3.1** | All six are `EXTERNAL`, non-`inline`, 1 parameter, non-varargs, so `index = s - 48*[leaf]` and the clause that fires is the **EXTERNAL `index <= 64`** arm. The asymmetry is therefore a **size** asymmetry: `?shuffle2`'s own emitted `.text` is smaller than the other five's. | optimistic |
| **P3.2** | Concretely: `?shuffle2` is a **leaf** and its standalone emitted size is `s <= 112` bytes (so `index = s-48 <= 64`); at least one of the declined five has `s > 112` **or** is non-leaf. | optimistic |
| **P3.3** | `n_sites(?shuffle2)` within `keygen_xbox.cpp` is **1**. (`N_max` is UNBOUNDED on the EXTERNAL accept arm, so this is not load-bearing for the decision — registered so that a site-count explanation can be *excluded*, not assumed.) | — |
| **P3.4** | The inlined 14 words are **not** `?shuffle2`'s emitted body spliced verbatim: the inlined copy is frameless and re-allocated, so `SPLICE-0` (`w-splice`) would be wrong here. I predict ≥ 1 word differs beyond a register field between `?shuffle2`'s own COMDAT and the 14 words inside `?supershuffle`. | optimistic |

## P4 — the grid

| # | registered |
|---|---|
| **P4.1** | The grid will separate a **size-of-callee** rule from a **caller-side budget** rule on ≥ 4 cells, and the size rule wins. |
| **P4.2** | The `/Ob0` control is categorical: **every** cell declines at `/Ob0`, including cells whose callee is `__forceinline`. |
| **P4.3** | The threshold **moves with favor-speed** (`/O1 /Ot` vs `/O1`, `/O2` vs `/O2 /Os`) exactly as the `memcpy` threshold did, on ≥ 2 cells. Registered **optimistic**; if the inline ceiling is the same 64 bytes at every flag set, P4.3 is retracted and the option-word half of P1.3 goes with it. |
| **P4.4** | `__forceinline` overrides the size ceiling at every size the grid tests, at every optimizing flag set, and is the one clause that is **not** the cost function. |

## P5 — what it is worth to the port (the pricing deliverable)

| # | registered | direction |
|---|---|---|
| **P5.1** | **A correct inline DECISION converts zero frontier TUs on its own.** The decision says *whether* to expand; the bytes then require the port to lower an arbitrary callee body **into a caller's register allocation** — WB-D's open question. `w-splice` already shipped the only case where the decision is free of that (SPLICE-0: the caller's body *is* the callee's body) and got 723 functions and **0 TUs**. | **PESSIMISTIC — the headline** |
| **P5.2** | For `?supershuffle` specifically, the priced remedy is **transcription** (a recognizer plus 26 hand-derived words for one function), and it converts `keygen_xbox.cpp` only if the other 19 emitted functions in that TU also come out exact — which #1474 says they do **not** (1 exact of 20). So the honest price of the anchor is **> 1 TU of work for 0 TUs of movement**. | pessimistic |
| **P5.3** | The one thing this lane could produce that is worth shipping is a **narrowing of `IlBundle::functions()`'s wholesale refusal** — i.e. a *safe* region of the decision (the categorical arms: varargs, recursion, `/Ob0`, and the `index <= 64` accept) rather than the cost model. Registered as the recommendation I expect to make. | — |

## Decline clauses, registered in advance

* If the grid shows a rule the disassembly reading contradicts, **the reading is
  retracted**, not narrowed (method doc §7).
* No cell of this lane's grid may be added to `fixtures/` and nothing is adopted
  into `crates/` under any outcome.
* If the cost routine cannot be located at all, this lane reports that as its
  result and the incumbent `INLINE-P` stands unchallenged; a *narrative* reading
  of a routine that is not obj-checked earns no DISCLOSURE row.

## Registered overall direction

**PESSIMISTIC.** The incumbent is already at 0.9716 from the outside; this lane
can plausibly explain constants and still move nothing. P2.1 is the one place a
miss would be in the *optimistic* direction and it is flagged as such.
