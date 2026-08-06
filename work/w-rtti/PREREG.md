# w-rtti — PREREGISTRATION

    Lane:    w-rtti
    Base:    master `9827bcf` (merge w-inline)
    Written: before any probe, any capture, and any `crates/` edit exists.
    Board:   #926–#935 reserved for this lane.

The mission is the factor-C ladder's top step: **teach the COFF writer
`.rdata$r`**, verifying `docs/OBJ_RDATA_R_SHAPE.md` against fresh captures
first, then shipping a writer the differential can grade, then measuring the
gap metric at both ends.

Lane `w-rdata` (2026-08-04) measured this step and **declined** it at seven
independent refusals, two of them in `crates/c2-il`. This lane is briefed to
ship it anyway. **That disagreement is the thing being tested**, so it is
registered as a claim that can lose, in both directions, below.

---

## §1 What is registered

Every row is a number I do not have yet. `factor-c` and friends are read from
`c2rs gap`'s own `gap-metric` lines, not from `docs/STATUS.md`.

### §1.1 The metric block, at both ends

| key | registered BEFORE | registered AFTER (if the writer ships) |
|---|---:|---:|
| `factor-c` | **169** | **590** |
| `b-and-c` | 151 | **315** |
| `a-and-b-and-c` | 27 | **27** — unchanged |
| `frontier` | 19 | **19** — unchanged |
| `ladder-head` | `.rdata$r` → 590 | `.text$yd` → 804 |
| `match` | **10** | **10** — unchanged |
| `mismatch` | 0 | **0** |
| `capture-fail` | 7 | 7 |

**P1.** `factor-c` reads **169** before. *Can lose:* master has moved 20+
commits since the 169 was last published in a rung; if a lane widened the
vocabulary the before-number is not 169.

**P2.** `factor-c` reads **590** after, if and only if the writer ships.
*Can lose:* the ladder is greedy and recomputes; `w-joint2` and `w-rdata` both
read 590 by two independent routes, but both were measured at an older master
and the workload stamp has changed (`940d07dc` → `fe1b5b39` → `798ae68c`).
**If the measured number is not 590 I report the measured one and the ladder's
projection is the thing that was wrong.**

**P3.** TU match is **10 → 10**. Registered as a *point mass at zero*, not an
estimate: `w-joint2` measured `|{A∧B} \ C| = 0` and `|D∨E| = 10` with 0 of the
676 `.rdata$r` TUs inside `D∨E`. **C is necessary, not sufficient.** *Can
lose:* if match moves at all — either direction — the §10 correction in
`OBJ_RDATA_R_SHAPE.md` is wrong and that is the lane's headline.

**P4.** The FBM partition is **untouched**: exact **34,466**, differs **4,711**,
partition-broken **0**, match-tu-differs **0**, scan mismatch **0**. This lane
emits no functions *unless* §2's caller requires a `.text` body, in which case
FBM's exact count may move and P4 loses on the first term only. Registered
separately: **partition-broken 0 and match-tu-differs 0 and mismatch 0 may not
move under any circumstance** — those three are the alarm, not the metric.

### §1.2 The spec-verification grid

`docs/OBJ_RDATA_R_SHAPE.md` was derived from a 22-source grid at
`/GR /O1 /Oi /EHsc /GS- /c` plus `/Od` and `/Ox`, and 38 real workload objs.
This lane re-checks it on **cells it was not derived from**.

**P5.** Every `.rdata$r` section in the fresh grid carries Characteristics
**`0x40301040`** and COMDAT Selection **2 (ANY)**, and every relocation in a
`.rdata$r` section or a `??_R0` `.data` COMDAT is **`IMAGE_REL_PPC_ADDR32`**
and nothing else. *Can lose:* the spec says "no alignment or selection
variation was observed" over its own cells; an 8-byte-aligned or `LARGEST`
record in a cell it never compiled refutes it.

**P6.** The **DFS pre-order** rule of §5 is exact on every fresh grid obj.
*Can lose:* the spec claims 25/25 and 38/38 with "no other rule was needed";
a fresh cell — deeper virtual inheritance, a template instantiation, a nested
class, `__declspec(dllexport)` — that violates it refutes the one rule the
whole emitter would be built on.

**P7.** The aux `CheckSum` of every fresh record reproduces under the port's
existing `coff_checksum` (reflected CRC-32, init 0, no final inversion).
*Can lose:* trivially, on any record.

**P8.** The `??_R0` TypeDescriptor is in **`.data`** (writable, `0xC0301040`),
unpadded, `8 + strlen(name) + 1`. *Can lose:* a long or a template name that
pads.

I expect at least one of P5–P8 to lose. A spec re-verified on fresh cells that
finds **nothing** is weak evidence that the cells were fresh.

### §1.3 The ship / decline decision — registered in advance

**P9 — the decline floor.** `PORT_WRITER_SECTIONS` gets `.rdata$r` **only if**
all three hold, and I commit to this before knowing whether they can:

1. a `Section { name: ".rdata$r", … }` literal exists in `crates/c2-core/src/coff/`,
2. it has a **real caller reachable from `PortC2::build`** — not a test, not a
   dead `pub fn`. Board **#278** deleted `container::bss_deferred_layout` for
   exactly this and `w-rdata` §4 declined rather than repeat it,
3. **the differential grades at least one real obj containing `.rdata$r`
   byte-exact** against real `c2.dll` under wibo, TimeDateStamp zeroed.

If any of the three fails, **the constant stays at 10 names, `factor-c` stays
169, and this lane reports the refusal with its price** — the same verdict
`w-rdata` reached, re-derived at a different master, which is a result and not
a failure to execute. **I register that this is a live possibility and not a
formality**, because §2 says the price is seven facts and I have not yet
checked whether any of the seven have been paid since 2026-08-04.

**P10 — the price.** `w-rdata` priced the minimal `.rdata$r` obj at **seven**
independent refusals (its own P3 registered *two* and was refuted at seven, in
the direction that made the work look cheaper). I register that **the price
re-derives at seven or more**, of which **at least two are `crates/c2-il`'s**.
*Can lose:* if lanes since 2026-08-04 have paid some of them, the count is
lower and the ship is cheaper than the decline clause assumed. That is the
single most important thing to check before writing any code, and it is
checked first.

**P11 — where it refuses.** Whatever ships, the port **refuses** (returns
`NotImplemented` / `None`, never a wrong obj) on every `.rdata$r`-bearing TU
outside the modelled class. Registered as an invariant, not a prediction:
`mismatch` must be 0 on every run, and `scripts/expr_sweep.sh` and
`scripts/mode_cross.sh` inside `gate.sh` must read 0 mismatch. There is no
outcome of this lane in which a wrong emit is acceptable.

### §1.4 Gate and tests

**P12.** `scripts/gate.sh --jobs 6` reads **18/18 PASS, 0 mismatch** at the
tip. `cargo test --workspace --release` reads at least the base's
`targets=/passed=/failed=` — baseline **916 passed / 0 failed / 28 targets**,
**re-measured at this lane's own base** before it is compared to anything.
*Can lose:* a target that aborts early reports a *smaller* passed count that
reads as green (`w-rdata` §7). FAILED is compared; PASSED is compared only
after the target count is confirmed.

---

## §2 The seven facts, as inherited

From `docs/rungs/2026-08-04-w-rdata.md` §4 and `OBJ_RDATA_R_SHAPE.md` §8.
Reproduced here so the re-derivation in §1.3's P10 has something to disagree
with, and so a lower count is visibly a *finding* rather than a redefinition.

| # | fact | crate |
|---:|---|---|
| 1 | the vfptr-store leaf body class (`expr-op-0x27`) | `c2-il` |
| 2 | a reader for the `??_R*` record graph | `c2-il` |
| 3 | codegen for `lis/addi/stw rD,0(r3)/blr` — a `DataRef` whose low half feeds a **store** | `c2-core` |
| 4 | the `.rdata$r` / `.data`-COMDAT `Section` emitter and its `ADDR32` relocations | `c2-core` |
| 5 | the DFS emission order over sections **and** undefined externals | `c2-core` |
| 6 | the vftable `.rdata` COMDAT: Selection 6, symbol `Value` 4 | `c2-core` |
| 7 | the `??_7type_info@@6B@` undefined external | `c2-core` |

---

## §3 Method, fixed in advance

1. Re-measure the base's metric block and test counts **first**, on this
   worktree, before any edit. Numbers quoted from `docs/STATUS.md` are a cache
   (its own first paragraph says so) and this lane quotes from a scan.
2. Freeze the spec-verification grid's source list **before** capturing, so a
   cell cannot be dropped after it disagrees. The grid goes in
   `work/w-rtti/grid/` with one `.gl` source directory (the `w-ilx` rule).
3. Re-derive the price (P10) before writing code.
4. Ship or decline per P9.
5. Re-measure the metric block, the FBM partition line **in full**, the gate,
   and the workspace tests.

Positive checks with printed counts; a totals line is not a control (traps 4
and 5). No judging output through `head`/`tail`. No positional readers (#644).
No glob or recursive walk of `work/capture-cache` or `.claude/worktrees`.
