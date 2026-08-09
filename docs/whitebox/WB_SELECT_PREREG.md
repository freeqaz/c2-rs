# WB-I `wb-select` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing here is
> adopted into `crates/`.

Registered **before the first grep of `~/ghidra-projects/export/c2/`** and
before the first `cl.exe` of this lane, per board #770's standing rule (running
estimate streak ~10 optimistic / 2 pessimistic / 1 hit). Scored in
[`WB_SELECT_FINDINGS.md`](WB_SELECT_FINDINGS.md) §8.

Lane: `wb-select` / branch `worktree-agent-a31667159c896762b`, branched at
master `c5ff9953`. Image sha256 verified at the top of this lane:
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
(`~/ghidra-projects/bin/c2dll`, matches method doc §0).

**What was read before freezing.** In-repo only:
`CAMPAIGN_2026-08-08.md` (all), `CAMPAIGN_2026-08-08_GENERATORS.md` (all),
`C2_MAP_METHOD.md` (all), `WB_REGALLOC_FINDINGS.md` §1–§5, §9, §10,
`WB_LOOP_FINDINGS.md` §0, §1, §6, §9, §10, `WB_LOOP_PREREG.md` (format),
`WB_INLINE_FINDINGS.md` §3–§3.2, `scripts/gt_capture.sh`, `scripts/gt_dump.py`
header, `docs/rungs/2026-08-08-wb-loop.md` header.

**Zero export greps, zero disassembly reads, and zero `cl.exe` invocations of
this lane's authorship had happened when the predictions below were written.**
The VAs quoted in the predictions are quoted *from the in-repo findings docs of
WB-D and WB-H*, not from the export.

**Inherited facts this lane treats as given** (each already obj-checked by the
lane that produced it, so re-deriving them is not this lane's job):

* WB-D §2 — the register-number table at `0x10b181c0`; §3.1 the GPR order;
  §3.4 the min-cost selector `0x10b2e7f8`; §4 "no scheduler exists"; §5 the
  operand-nibble→class map `0x10b022cc`.
* WB-H §0 — the counted-loop normal form is three independent passes; `lur.c`
  is READ and CLOSED. This lane does not re-tread it.
* WB-H §9 item 3 — *"the pattern set for the body's operators: STILL NO, and
  it is now the only 'no' left"*, and its narrowing observation that within a
  converted loop body every one of 36 cells was a plain one-to-one lowering.
  That narrowing is a **prediction about the pattern set** made by another
  lane; P4.1 below scores it.

---

## P0 — the success floor

| # | prediction | direction if wrong |
|---|---|---|
| P0.1 | This lane clears its floor: at least one pattern-set reading survives a frozen obj-check on an idiom the port does not emit today (the branchless carry compare is the intended one). | optimistic |
| P0.2 | The branchless carry idiom for `x <u K` is **not a peephole over a compare-and-branch**; it is selected directly, because the value-producing context is known at selection time. | — |
| P0.3 | The lane's *judgment* deliverable comes out **"yes, an arbitrary straight-line body is lowerable from a derived pattern set"**, with the pattern set for the operators the IL actually carries being **under 120 rules**. | optimistic |

## P1 — where instruction selection lives

| # | prediction | direction if wrong |
|---|---|---|
| P1.1 | Selection is **table-driven at least in part**: there is an image-resident array indexed by (IR opcode) or (IR opcode × type) that yields a PPC opcode or an expander function pointer. | optimistic |
| P1.2 | `p2\ppc\cgintrin.c` (string `0x10b19698`, xrefs in the `0x10bf0…0x10bf9…` band per WB-H's PREREG disclosure) is **not** the main operator selector — the name says *intrinsics*, and it will turn out to hold `memcpy`/`memset`/`__builtin` expansions and the compiler-generated helper calls, not `+`/`<`/`<<`. | pessimistic (i.e. if wrong, the lane's entry point is easier than predicted) |
| P1.3 | The main per-operator selection lives in **`p2\ppc\lower.c`** (`0x10c053e7` band) and/or `p2\ppc\lowersmd.c` (`0x10c23539`), as a `switch` on the tuple opcode with one arm per operator family. | — |
| P1.4 | The **final expansion switch** WB-D §4 saw (the in-place pseudo-op rewrite reached from `0x10c216f5` / `0x10c21719`) is a **different, later** pass than operator selection: it expands *pseudo-ops* (prologue, epilogue, calls, large-constant materialisation), not `+`/`<`. Its dispatch is a **jump table** in `.rdata`, and this lane will name that table's VA. | — |
| P1.5 | There is at least one image-resident table of **PPC opcode encodings/mnemonics** (a mnemonic string array, like the register-name array at `0x10b181c0`) whose index is c2's internal machine-opcode number, and it is the Rosetta stone that makes the selection arms readable. | optimistic |
| P1.6 | The tuple opcode numbers WB-H recorded (`0x2d4` compare, `0x2af` assign, `0x2c6` add, `0x288` branch, `0xf8` ctr-decrement) are **dense within one contiguous numeric band** for the arithmetic/compare operators, i.e. the selector can switch on `op - base`. | — |

## P2 — the pattern set's SHAPE

| # | prediction | direction if wrong |
|---|---|---|
| P2.1 | The pattern set is **not** a generic bottom-up rewriter (no BURG/iburg cost table). It is hand-written C: a `switch` per operator with `if`-ladders on operand kinds (constant? fits in 16 bits? power of two?). | — |
| P2.2 | Constant operands are tested against a **16-bit signed** fit to choose the `-i` immediate form (`addi`/`cmpwi`/`ori`), and against a **`u16`** fit for the logical forms (`ori`/`andi.`), matching PPC's asymmetric immediate encodings. | — |
| P2.3 | Integer `*` by a constant is strength-reduced to shifts/adds below some population-count or magnitude threshold, and to `mullw` above it. | optimistic |
| P2.4 | Integer `/` and `%` by a non-zero constant are lowered to the **magic-number multiply** (`mulhw`/`mulhwu` + shifts), and the magic-number computation is a routine in the image (Granlund–Montgomery), not a table. | — |
| P2.5 | There is an idiom recogniser that fires on a **combination** — at minimum, `(a << k) + b` folding, and `x != 0`/`x == 0` → `cntlzw`+`srwi`. | optimistic |

## P3 — the branchless carry idiom (the flagship)

Stated cold, before any read. WB-D §7/§9.1 reports that `x < 10u` returning
`1` or `2` lowers to a **four-word branchless `subc`/`subfe` carry idiom**.

| # | prediction | direction if wrong |
|---|---|---|
| P3.1 | The mechanism is: a subtract that **sets XER[CA]** (`subfc`/`subfic`, extended-mnemonic `subc`), followed by `subfe rD,rD,rD` which computes `¬rD + rD + CA = CA − 1`, i.e. a **0/−1 mask**, followed by one arithmetic fixup (`addi`/`subfic`/`neg`/`and`) to map the mask onto the two result values. | — |
| P3.2 | The idiom is selected only when the comparison is **unsigned** (or provably non-negative) — a **signed** `<` producing a value does **not** get it, because the signed predicate is not a carry-out. Signed value-producing compares get either `srawi`+`srwi` (for `x < 0`) or a compare-and-branch / `subfe`-free sequence. | optimistic |
| P3.3 | The idiom generalises over the two result values: `?:` with arbitrary constants `A`/`B` uses the same mask and differs only in the fixup, provided `A−B` is a 16-bit immediate or a power of two (`and` with a mask). | optimistic |
| P3.4 | The idiom is **not** taken when the comparison feeds a **branch** — branch context selects `cmplwi` + `bc`, per #1788. The selector therefore has a **value-vs-branch context bit**, and this lane will name where it comes from. | — |
| P3.5 | `x < K` where `K` is a **variable** (not a literal) also gets a carry idiom, using `subfc rD,rA,rB` instead of `subfic`. | optimistic |
| P3.6 | The idiom's presence is controlled by an optimisation level or a `-QX` switch, so it can be isolated by a counterfactual recompile (WB-H §7.7's method). | optimistic |

## P4 — the rest of the operator set

| # | prediction | direction if wrong |
|---|---|---|
| P4.1 | WB-H §9 item 3's narrowing **holds**: integer `+`, `−`, `*`, `^`, `&`, `\|`, and the shifts are one-to-one lowerings with no idiom library involvement, so the pattern set for them is a table and not an algorithm. | — |
| P4.2 | Signed `>>` by a constant is `srawi` **and** requires nothing else; signed `/` by a power of two additionally emits the sign-bias `addze` (`srawi`+`addze`), which is a 2-word idiom the port does not emit. | — |
| P4.3 | `char`/`short` narrowing on assignment emits `extsb`/`extsh` (WB-H saw `extsb`), and `unsigned char` narrowing emits `rlwinm` (`clrlwi`), not `andi.`. | — |
| P4.4 | Record-form (`.`) instructions are emitted **only** where a compare against zero immediately consumes the result, and c2 will turn out to have a peephole that fuses `op` + `cmpwi rX,0` into `op.` — i.e. record forms are a *fusion*, not a selection. | optimistic |
| P4.5 | The count of distinct selection arms for the integer scalar operator set (everything except float, VMX, calls, and intrinsics) is **under 120**. | optimistic |

## P5 — the judgment (deliverable 4)

| # | prediction | direction if wrong |
|---|---|---|
| P5.1 | The answer to *"can the port lower an ARBITRARY straight-line body from a derived pattern set"* is **YES with a named boundary**, and the boundary is set by the **idiom recognisers** (combinations), not by the per-operator table. | optimistic |
| P5.2 | The **first general class** to attempt is `expr_straightline_int` — a single basic block, integer scalar operators only, no calls, no memory beyond loads/stores of locals and parameters — and its predicted reach on the 124-TU reach-pool is **0**, for the same reader-gate reason WB-D P5.4 and WB-H §9.1 both scored (48/59 frontier functions die at the port's IL reader). | — |
| P5.3 | A `lower_expr` general path is **smaller** in port terms than the four transcribed body shapes it would subsume — under 800 lines of Rust for the integer operator set. | optimistic |
| P5.4 | This lane will find **at least one** operator whose lowering it can read but **cannot predict** without a further unread pass (the WB-D `M2` shape of result), and it will name it rather than paper over it. | — |

## P6 — grid mechanics

| # | prediction | direction if wrong |
|---|---|---|
| P6.1 | At `/O1` the graded cells survive constant folding as long as every operand comes from a parameter — the wb-inline v1 failure mode does not recur. Checked by a calibration pass that reads **section sizes only** (`gt_dump.py --no-disasm`), never a word sequence, before the graded predictions are frozen. | — |
| P6.2 | At least **2** of the graded cells will MISS. A grid on an unread pattern library that scores 100% would mean the cells were too easy. | pessimistic |
| P6.3 | Register choice inside the graded cells follows WB-D §3.4 unchanged (`r11` first for scratch, `r3` for the return by copy preference), so a word-exact miss will be a *selection* miss and not a register miss. | — |

---

## What would make this lane a failure

Not "the predictions missed" — misses are the point of freezing. Failure is:

1. no reading of the pattern set at all (the selector turns out to be spread
   across passes with no locatable table), **and** no written finding of why; or
2. a grid whose cells cannot separate "read the pattern set" from "guessed the
   obvious PPC lowering" — i.e. every cell is one the port already emits.

Guard against (2) is asserted before the run: **at least 4 graded cells must
predict a word sequence that `c2-core` does not emit today**, and at least 2
must involve a *combination* of operators rather than one.
