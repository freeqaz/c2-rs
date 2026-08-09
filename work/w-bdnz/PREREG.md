# w-bdnz — PREREG

**Frozen and committed BEFORE the first change to `crates/` and before the first
fixture this lane authors.** Lane `w-bdnz`, worktree branch `wt-w-bdnz` off
master `e253ee0e` (the w-mcall merge). Board rows **#1980**–**#1999**.

The commission: ship the counted-loop class of ROADMAP §10.26.1 / §10.26 item 4
— `wb-loop`'s **first two** of three composable passes, the rotated pre-test
guard and the `mtctr`/`bdnz` conversion — as a recognizer in the PARSER and a
lowering in the emitter, byte-exact against real `c2.dll` on manufactured cells.
The **update-form pass is DECLINED BY NAME** (decline D9 below): `wb-loop` §4.4
left its selection rule explicitly undecided at RU0′ 8/10 vs RU2 8/10 on
disjoint cells and filed RU-H as an unfrozen hypothesis. This lane does not
attempt it and does not ship a cell that needs it.

---

## §0 — the base, re-derived rather than inherited

Six inherited survey prices have been wrong this week (STATUS.md's own record),
so nothing below comes from a rung, a board row or the commission. Every number
here was produced by a command in this tree at `e253ee0e`, logged under
`work/w-bdnz/`.

| fact | value | source |
|---|---:|---|
| workload scan | match **18** · mismatch **0** · codegen-gap **0** · vocab-gap **853** · port-error **0** · capture-fail **7** | `scan_base.out` |
| `gap-metric` keys | **251** | `scan_base.out` |
| function census | **711514** / 2463443 (28.88 %) | `scan_base.out` |
| emitted census | **39200** / 178977 (21.90 %) | `scan_base.out` |
| `fnbyte-exact` · `fnbyte-differs` | **36228** · **2111** | `scan_base.out` |
| factor A/B/C · `b-and-c` · `a-and-b-and-c` · FRONTIER | 28 / 338 / 169 · 151 · 27 · **9** | `scan_base.out` |
| `progress-mass` | **0.20831** | `scan_base.out` |
| tracked fixtures | **309** | `fixall_base.txt` |
| fixtures at `/O1` | match **155** · mismatch 0 · codegen-gap **9** · vocab-gap **145** | `fixbase.out` |
| fixtures at `/Ox` | match **140** · mismatch 0 · codegen-gap **17** · vocab-gap **152** | `fixbase_ox.out` |
| workspace tests | **1335 passed, 0 failed, 36 targets** | `tests_base_full.txt` |

The three body counts and the key count reproduce w-mcall's published tip digit
for digit, so the base really is the tree that rung was written against.

### §0.1 The class's own base population, measured not guessed

A counted-`for` body reaches the parser and **decodes end to end** — it is not a
decode failure. `c2rs census` on `work/w-bdnz/probe/L1.cpp` and `L2.cpp`:

```
GAP expr-jump   cflow-loop   eh-none      ?sum@@YAHPBHH@Z
  1 x expr-jump   … 32 86 41 74 4b >3a< e8 09 29 e9 09 …
```

Every cell of this class blocks at the **`3A` unconditional jump** of the `for`
rotation, under key `expr-jump`, having decoded as `cflow-loop`. So the accept
population this lane can reach is a subset of `expr-jump`, and the honest
statement of the ceiling is the workload's `expr-jump` count — **not** a claim
that the class is that size (w-mcall §4.3: a first-blocker key names what
stopped the parse, not what the body is; five lanes have now been dispatched off
such a ranking and found it an artifact). `expr-jump` at base is
**2,286 bodies / 302 emitted** (`work/w-bdnz/keys_base.txt`); **that is
registered as an arithmetic ceiling and the real class is a strict subset of
it.**

---

## §1 — THE CLASS, in port terms, with every clause's exercising cell

At `/O1` **and** `/Ox` (both graded — see §1.3):

```c
    T f(int n, int k) {          // exactly two formals; slot 0 is the BOUND
        int s = INIT;            // INIT a simm16 literal; the accumulator
        for (C i = 0; i < n; ++i)   // start 0, step +1, relation `<`
            s OP= k;             // ONE compound assignment, operand = formal 1
        return s;                // the loop is the function TAIL
    }
```

with `OP ∈ { -=, *=, &=, |=, ^=, <<=, >>= }` and `C` the counter's type,
`int` or `unsigned` — **and the bound's type must equal the counter's**.

Emitted (eight words, read off real `c2` in `work/w-bdnz/probe/L3.obj` and
`L5.obj`):

```
    mr     r11, r3
    li     r3, INIT
    cmp{w,lw}i cr6, r11, 0        <- the ROTATED PRE-TEST GUARD (pass 1)
    bclr   {4,25 | 12,26}         <- realised as a CONDITIONAL RETURN
    mtctr  r11                    <- the TRIP COUNT      (pass 2)
    <OP>   r3, r3, r4
    bdnz   .-4                    <- the LATCH           (pass 2)
    blr
```

`sub` is `subf r3,r4,r3`; the other six are `mullw/and/or/xor/slw/sraw
r3,r3,r4`.

### §1.1 The eight boundary clauses of `wb-loop` §5, each with its cell

| # | clause | positive cell | NEGATIVE cell (measured, not assumed) |
|---|---|---|---|
| 1 | single back edge | every P cell | `n_cont` — `continue` adds an edge into the latch. **c2 still converts** (`bdnz` with an inner `cmpwi/bt`); the port refuses |
| 2 | single exit | every P cell | `n_break` — c2 emits **no `bdnz`** (`addic./bf 2`), `wb-loop` `a1`/`a2` |
| 3 | 32-bit integer local counter | P1–P9 | `n_i64` — `long long i`: c2 emits `cmpd`/`bt 24`, **no `bdnz`** (`wb-loop` `a9`, its own MISS) |
| 4 | constant step, and REQUIRE ∈ {+1,−1} | every P cell (step +1) | `n_step2` (`i += 2`: c2 converts with `addi −1`/`srwi 1`/`addi +1` trip arithmetic — **unread**, so refused) and `n_stepv` (`i += k`: c2 emits no `bdnz`) |
| 5 | loop-invariant SYMBOL bound, not a computed expression | every P cell | `n_bexpr` — `i < n/2+3`: `srawi`/`addze`/`addic.`, **no `bdnz`** (`wb-loop` `a10`) |
| 6 | counter used ONLY by the exit compare | every P cell | `n_ctru` — `s *= i`: **c2 converts** and keeps a second `addi 11,11,1`; the port refuses |
| 7 | body free of calls / computed branches / CTR-taking inner loops | every P cell | `n_call` (`s *= f(k)`: framed, `bl`, no `bdnz`) and `n_nest` (inner takes CTR, **outer** gets `addic./bf 2`) |
| 8 | the body is ONE basic block | every P cell | covered by `n_cont` and `n_break` above |

**Four clauses beyond `wb-loop` §5, each forced by a measured re-plan and each
with its own cell** — these are this port's, not `c2`'s:

| # | clause | NEGATIVE cell |
|---|---|---|
| 9 | the bound is formal **slot 0** | `n_swap` — `f(int k, int n)`: `c2` re-plans (`li r11,1` / forward `bf 25` / closing `mr r3,r11`), a different block plan |
| 10 | the loop is the function **tail** — nothing after it | `n_after` — `return s + 7`: the accumulator stays in r11 and the guard becomes a forward branch |
| 11 | the accumulate operand is a **formal**, not a literal | `n_litop` — `s *= 3` → `mulli`; and `s += 3` / `s -= 3` are **folded to `mulli` with no loop at all** |
| 12 | `OP` is not `+=` and not `/=` | `n_addop` (`s += k` → `mullw`, **the loop is deleted**) and `n_divop` (`/=` is a different spine: `rotlwi`/`divw`/two `twi`) |
| 13 | `INIT` fits `simm16` | `n_initover` — `32768` becomes `lis`/`ori` **and the guard compare interleaves between them** |
| 14 | exactly two formals, both `int`-like | `n_three` — a third formal. c2 emits byte-identical text; the port refuses anyway (the safe direction), and saying so is the point |

### §1.2 FENCE ORDER, frozen before the code

The recognizer is placed **LAST in `parse_segment_shape`'s `0x26` arm**, after
`ptr_walk_loop`, `if_call_join`, `static_scan_loop`, `ptr_walk_chain_loop`,
`store_run_bind`, `xlrc_create_guard` and `json_utf8_copy`, and immediately
ahead of `try_parse_assign_body_detail`.

*Why last and not "the order is free":* the four existing loop classes each
argue disjointness at the second statement. This class's second statement is
`53` then a **literal assignment to an `int` local** — which is one `53` away
from `xlrc_create_guard`'s stated separator ("a second literal ASSIGNMENT").
Rather than assert a disjointness this lane has not proved, the recognizer goes
last, so **no body any earlier production accepts today can move**, by
construction and not by argument. The only population it can reach is one every
production above declined and which `try_parse_assign_body_detail` refuses at
its first `3A`.

The recognizer is **non-committal**: its own cursor, `Err` on the first byte
outside its grammar, no `prod_tag`, no `disp` on decline. A declining body keeps
`assign`'s blocker and no census key moves.

### §1.3 The mode gate is asked in the PARSER (#1638) and again in the emitter

Board #1638's defect is a gate that lives only in the emitter: the census then
counts a function in class that `PortC2` refuses. The optimization word is read
**first, before any body byte**, exactly as `static_scan_loop` does.

`/O1` **and** `/Ox` are both accepted, and both are graded on every cell —
measured, not inherited: `work/w-bdnz/probe/L5ox.obj` shows `/Ox` emitting the
**identical eight words** for both the signed and the unsigned cell (packed into
one `.text` rather than two COMDATs, which is section layout and not codegen).
`/Od` refuses through the mode word as every class does. This is a departure
from `ptr_walk_loop`/`static_scan_loop`, which refuse `/Ox` because their lanes
graded no `/Ox` cell; this lane grades one per cell, at both modes, and the 18
gate lanes grade the rest.

### §1.4 The signedness trap (#1788) is the class's sharpest fence

The counter's `int` and `unsigned` spellings differ **only in the IL TYPE byte**
— `86 41 74` vs `86 42 75`. The relational opcode (`22`) and the branch (`38`)
are byte-identical (`work/w-bdnz/il_L5`, bodies 1 and 2). `readers::eat_int_like`
accepts BOTH, so a recognizer built on it would emit `cmpwi`/`bf 25` for a loop
whose obj has `cmplwi`/`bt 26` — **four wrong bytes in an obj that links**.
The recognizer therefore reads the type through `read_type` and requires
`kind & 0x0F` to agree on the counter's declaration, on both operands of the
compare, and on the `+= 1`.

---

## §2 — DECLINE CLAUSES, each named and sized

| # | declined | size / why |
|---|---|---|
| **D9** | **the update-form pass** (`lwzu`/`stwu`, and the base-difference X-form for ≥2 same-stride arrays) — declined **by name**, as the commission requires | `wb-loop` §4.4/§7.5: RU0′ 8/10 and RU2 8/10 on **disjoint** cells, RU0′-b retracted, RU-H unfrozen. The class is defined to contain no memory reference at all, so the pass cannot apply to any cell here |
| **D10** | the **trip-count arithmetic** for a non-unit step | `wb-loop` §9 item 4: the `divwu`-vs-`srwi` selector is unread. Cell `n_step2` |
| **D11** | any loop that is **not** the function tail (forward-branch guard) | cell `n_after`; a different block plan, not a different field |
| **D12** | a **literal** accumulate operand (`mulli`, `xori`, `andi.`…) | cell `n_litop`; each immediate form has its own selection rule and `andi.` writes CR0 |
| **D13** | `start != 0`, `<=`, `!=`, and the descending form | each is a different guard immediate and/or a `addi r11,r11,±k` before `mtctr`; measured in `work/w-bdnz/probe/L4.obj` (`start3`, `le`, `ne`, `down`) and **not shipped** |
| **D14** | an **unsigned accumulator** | `>>=` on `unsigned` is `srw`, not `sraw` — a different word keyed on a type this lane does not carry. Cell `n_uacc` |
| **D15** | widening `PORT_CFG_CLASSES`, `IlBundle::functions()`, any whole-TU recognizer, any `DISCLOSURE.md` adoption | held |
| **D16** | any `label_slots` value other than `None` for this shape | §3 |
| **D17** | more or fewer than two formals, and any non-`int`-like formal | cell `n_three` |

---

## §3 — The label counter: `None`, and the lead is MEASURED not inherited

`ptr_walk_loop`, `ptr_walk_chain_loop` and `static_scan_loop` all return `None`
from `label_slots` because `w-loop` measured that a leaf loop charges `+1..+4`
and that *which* of the four cannot be read off the emitted bytes. This class
inherits that argument whole and returns `None` (D16), so `IlBundle::functions`
refuses any TU pairing it with a framed function and admits one that does not.

**The commission requires the lead to be MEASURED against the obj rather than
read off `LABEL_COUNTER.md`** — w-json measured its §1.1 surcharge two low for a
back-edge class. The measurement is a two-TU counterfactual in w-json's form:
one TU is `<this class> ; <framed fn>`, the control is
`<leaf-none> ; <framed fn>`, and the lead is the difference in the framed
function's own `$M` number as real `c2` mints it.

**Registered prediction (P11):** `LABEL_COUNTER.md` §4.2.1's `for` row says a
leaf `for` loop charges **+2** over `leaf-none`'s 1, i.e. an absolute charge of
**3**. Registered at p = 0.45 that the measurement is exactly 3, and at p = 0.80
that it is in `[2, 5]`. **If it is not 3 the table is wrong for this class and
the rung says so** — the `None` does not move either way.

---

## §4 — PREDICTIONS, in probability form

| # | prediction | p |
|---|---|---:|
| **P1** | `mismatch` **0** everywhere: 878 TUs, all fixtures at `/O1` and `/Ox`, all 18 gate lanes, the sweep and the cross | 0.90 |
| **P2** | TU match **18 → 18** | 0.93 |
| **P3** | FRONTIER **9 → 9**, same members by name, per-TU key histograms byte-identical | 0.88 |
| **P4** | **workload conversions: 0.** The registered expectation, #1829/#1921's shape — this is infrastructure and is priced as infrastructure | 0.85 |
| **P5** | the workload **function census moves by 0** | 0.70 |
| P5b | the workload **emitted census moves by 0** | 0.80 |
| **P6** | `gap-metric` key count stays **251** | 0.85 |
| **P7** | `fnbyte-differs` does not rise (**2111 → 2111**) | 0.85 |
| **P8** | the positive fixture is a whole-TU `match` at `/O1` **AND** at `/Ox` | 0.70 |
| P8b | …and at all 18 gate lanes that map to `OptMode::O1`/`Ox` | 0.65 |
| **P9** | no pre-existing fixture moves at either mode | 0.75 |
| **P10** | test DELTA **+16 exactly** (registered as a DELTA per #1749, and the tests are checked BY NAME, not by subtracting totals) | 0.35 |
| P10b | the delta is in `[+10, +24]` | 0.80 |
| **P11** | the label lead measures **3** (see §3) | 0.45 |
| P11b | …in `[2, 5]` | 0.80 |
| **P12** | ≥ 1 **unnamed refusal** — budgeted one, and PRE-ARMED on two places: (a) FENCE ORDER, that a production placed last still moves a census key because an earlier production's *decline path* has a side effect; (b) CLAUSE REACHABILITY, that one of the 14 clauses above is unreachable because an earlier clause already excludes it, so its `_neg` cell probes a key this lane does not own | 0.80 |
| **P13** | every one of the 14 `_neg` cells reports a **distinct** clause, probe-verified per cell before the fixture is written | 0.55 |
| **P14** | the `expr-jump` key falls by **0** on the workload (the class exists, and nothing on the workload is in it) | 0.65 |
| **P15** | `codegen-gap` on the 878 stays **0** — the parser and the gate agree (#139/#1638) | 0.85 |
| **P16** | **no DISCLOSURE row is needed.** W-LOOP-1 and W-LOOP-3 have complete black-box alternatives (the `wb-loop` grid cells + `/d2QXnobdnz`); W-LOOP-2's tuple opcode numbers and table `0x10b18990` are **not** consulted — this recognizer works on the port's own IL byte vocabulary (`26`/`33`/`3A`/`29`/`38`/`22`/`0F`…) and states clause 3 as "the counter is a 32-bit integer", which `wb-loop` §10 explicitly says needs no row | 0.90 |

**Board #770's streak.** Seven of `wb-loop`'s twelve misses were registered
optimistic; w-mcall added four more; all five of w-mcall's misses were the same
belief. P4, P5, P5b and P14 are registered **pessimistically on purpose** — the
commission registers ~0 reach and this lane does not talk itself into a number.

---

## §5 — Verdict neutrality, at three levels, and how each is checked

1. **878 TUs by name** — `work/w-bdnz/verdicts.py` over base and tip `--jsonl`,
   comparing the map `src -> (class, fn_in_class, fn_total)`. A count can hide
   one TU lost and one gained.
2. **All `gap-metric` keys accounted** — `work/w-bdnz/metricdiff.py`, a
   key→value MAP: vanished, appeared, changed and identical all counted.
   Never a `diff`.
3. **All 309 pre-existing fixtures at `/O1` AND `/Ox`, by name** —
   `verdicts.py` over the fixture scans.

Plus the first-blocker key map (`keydiff.py`) over both populations, whose
totals must be equal on both sides.

## §6 — Gate

`scripts/gate.sh --require-graded --jobs 12` at the shipping tree; beside it
`cargo test --workspace --release`, `scripts/board_audit.sh`, and
`cargo test -p c2-harness --release --test rung_registry`. Hatch needles per
w-park's precedent (`hatch-red`, `ladder-red`) on their first run.
