# `docs/whitebox/ref/` — the address-indexed reference for `c2.dll`

**Start here.** This directory is a **reference**: you arrive with an address
or a subsystem and leave with what is already known about it, plus the exact
command to go read the bytes yourself. It is deliberately *not* a findings
archive — the 19 `WB_*_FINDINGS.md` documents beside it are that, they are
dated, they stay as written, and this reference points **at** them rather than
restating them.

> **Whitebox analysis is authorized, encouraged, and not a legal risk**
> (`CLAUDE.md`, project owner, 2026-08-17). Byte listings, decompiled bodies,
> address maps and structure layouts are a resource worth building deliberately.
> The port stays I/O-behavioral — `port(IL) == c2(IL)` byte-exact is still the
> sole judge — because c2's own instruction bytes are the **wrong artifact**, not
> because reading them is off-limits.

Every address is an absolute VA in exactly

```
compilers/X360/16.00.11886.00/c2.dll
sha256  c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
size    1 347 072
```

**Verify that sha256 before trusting a single address here.**

---

## 1. The five questions, and where each is answered

| you have | you want | do this |
|---|---|---|
| **an address** | is anything already known about it? | `grep -P '^10b2e7f8\t' docs/whitebox/ref/ADDR.tsv` — containing function and size, callers/callees, translation unit and how far to trust that, subsystem page, hand label, and **every doc that already mentions it** |
| **an address `ADDR.tsv` has never heard of** (89 % of the image) | anything at all: where am I, who calls me, what do I touch? | `grep -P '^10b3540c\t' docs/whitebox/ref/FUNCS.tsv` — **one row per function in the image**, TU + confidence, coverage state, degree, hop distance to the nearest covered function, and the string literals and imports it references. See §4.1 |
| **a subsystem** | which functions, and what do they do? | [`SUBSYS.md`](SUBSYS.md), then the page it names |
| **a page's function** | the full decompiled body / raw bytes | §5, the retrieval recipe — the export regenerates in minutes and is deliberately not committed |
| **a behaviour** ("what sets the alignment nibble?") | the rule, with its provenance | §3 below, then the named page section |
| **whether you may use it** | | [`../DISCLOSURE.md`](../DISCLOSURE.md). **Navigation is free; adoption is not.** |

---

## 2. The provenance legend — the resource's main quality signal

Every claim on every page carries one of three marks. **An unmarked claim is a
defect.**

| mark | means | how far to trust it |
|---|---|---|
| **`[R]`** | **read** from the disassembly, and *not* checked against any obj or listing | **a hypothesis.** `[R]` says *"the instructions were read correctly"* — it does **not** say *"this is what c2 does"* |
| **`[O]`** | **obj-confirmed**: reproduced against real `c2.dll` under wibo, or against a `/FAsc` listing. The grid or cell is named | as good as a fact **on the fixture's structural coverage** — the fixture is part of the claim |
| **`[O] port`** | stronger: the port emitted a **byte-exact obj** on this reading | the highest status in this directory |
| **`[I]`** | an **inferred** step on top of an `[R]` or `[O]` | trust the premise, weigh the step |

**Why the distinction is not bureaucracy.** `C2_MAP_METHOD.md` §7 prices it:
the `.bss` bump rule was marked `high`, read correctly out of a small, clean,
fully-decompiled function — and it was **wrong about c2**. A function can be
present, correct-looking and simply **not on the path your inputs take**, or
guarded by a condition you did not vary, or overwritten downstream. Only the
oracle can tell you which.

And a second failure mode, from the emit predicate: a claim can be obj-checked,
reproduce perfectly, and still be **wrong as stated**, because the fixture
lacked the structure that would expose the gap. Six mutually-independent leaf
functions cannot distinguish *"this bit decides emission"* from *"this bit seeds
a queue that is then closed under reachability"*. **State what you tested on.**

### 2.1 Corrections are amended beside, never rewritten in place

`WB_DAGORDER_FINDINGS.md`'s revision-box rule is this directory's rule. A
document that silently absorbs its own corrections is one nobody can grade. Each
page's ⛔ boxes carry the corrections the record already contains, with the
original claim intact next to them.

---

## 3. Questions this reference answers — with the one lookup that answers each

Three of these are the usability test this lane registered in advance
([`PREREG.md`](PREREG.md) §4): questions past lanes answered the hard way.

| # | question | the one lookup | answer in one line |
|---|---|---|---|
| **U1** | Given an object of size *n*, what alignment nibble does c2 put in the section's `Characteristics`, and where is that decided? | [`P_SECTION.md`](P_SECTION.md) **§2** | `align(obj) = max(t, 1 if n<2 else 4 if n<64 else 8)`; **max** into `sect+0x43`; `(log2(a)+1)<<20` at `0x10b28261`; OR'd in by `0x10b289fd` **only when the IL override carries no nibble**. Measured table: `1→ALIGN_1, 2→ALIGN_4, 4→ALIGN_4, 8→ALIGN_8`, thresholds at `n=1→2` and `n=63→64` |
| **U2** | When does the section emitter write `Selection` at all, which values occur, and what computes the aux `CheckSum`? | [`P_COFF.md`](P_COFF.md) **§4** | `Number`/`Selection` are written **only when `Characteristics & 0x1000`** (`0x10b2948b`). `0`/`2` in `.data`/`.bss`, `1` in `/Gy` `.text`, **`5` (ASSOCIATIVE) in `.pdata`** — "only two values occur anywhere" is a statement about `.data`/`.bss`, not the obj. `CheckSum` is reflected CRC-32 `0xEDB88320`, init 0, no final XOR — and it is **not in `c2.dll`**, it arrives through the callback table at `0x10c44bf4` |
| **U3** | When two colouring candidates tie on priority, what decides? | [`P_REGALLOC.md`](P_REGALLOC.md) **§4** | `cand+0x44`, **descending, unsigned**, compared `<=` so an exact tie in both keys puts the newest candidate first (`0x10b2b82d`). **That field is not in `WB_LIVE_FINDINGS.md`'s enumeration at all**, and at `/O1` most cells are ties |
| | Is there an instruction scheduler? | [`P_DAG.md`](P_DAG.md) **§1** | **Yes**, run four times per function at `/O1`. `#1823`'s "there is no scheduler" is refuted; its band is a TU with no ICE site, so `c2_tus.tsv` cannot see it |
| | What decides whether a function body is emitted? | `C2_MAP.md` §3E, and §6.2 below | the `0x20` bit at `sym+0x4c`, arriving **verbatim from the IL**, closed under "referenced by an already-emitted function". Outside `-optref` c2 never subtracts |
| | Where does the inline decision live and what are its numbers? | [`P_INLINE.md`](P_INLINE.md) **§2–§3** | `0x10b5fb5f` candidacy, `0x10b60930` accept/decline; the size test is **skipped entirely** when the favor-speed bit `0x10c2e310` is set. **§2.1's four addresses are CORRECTED** — they were in `FUN_10b5fcd8`; the real test is `0x10b5fc7e`–`0x10b5fc90` |
| | **What size does the inliner actually measure, and can the port read it?** | [`P_INLINE.md`](P_INLINE.md) **§2.1a–§2.1c** | `[sym+0x50]` is the **`.gl` function record's `SIZE` field**, read verbatim by `il-read-varint16` at `0x10b9bf6c` — the field `gl_function_attrs` already walks past to reach `ATTR`. **It is an upper bound, not the tested value**: two callees with `SIZE = 115` get opposite verdicts, because folding reduces it before the inliner looks. Sound one-sided form: `SIZE < T ⇒ c2 inlined it`. `/O1` `T = 98`, `/Ox` `T = 122` |
| | How is the `.pdata` unwind word computed? | [`P_EH.md`](P_EH.md) **§2.2** | `(hasHandler<<31) \| (1<<30) \| ((len_words & 0x3FFFFF)<<8) \| (prolog_words & 0xFF)` at `0x10bff811`, patched in a deferred pass |
| **U4** | How does c2 realise a `.gl` alias in the obj, and in what order? | [`P_SYMBOL.md`](P_SYMBOL.md) **§2** | a **COFF weak external pair**: the target's own `EXTERNAL` record first, minted on demand at `0x10b28ce1` **because the emitter recurses into the target at `0x10b28cb9`**, then the alias at `StorageClass 0x69` with aux `{TagIndex = the default's index, Characteristics = 2}` (`0x10b28cfd`, `0x10b28cec`, `0x10b28cea`). There is a **second route** in that no alias-keyed grid can see: `[sym+0x3f] != 0` at storage kind 2 (`0x10b28c7d`) |
| **U5** | What decides whether a symbol gets a section, a section number of 0, or no COFF record at all? | [`P_SYMBOL.md`](P_SYMBOL.md) **§3** | the storage-kind field `([sym+0x37]>>5)&0xF` at **`0x10b28be6`** — a four-way `dec`-chain. And **two suppressions**: `[sym+0x32]&1` (already written) and the 3-bit linkage field `([sym+0x37]>>0x15)&7 ∈ {1,3}` at `0x10b28bb4`/`0x10b28bbd`, which writes nothing at all |
| **U6** | What `Type` does c2 put on a symbol? | [`P_SYMBOL.md`](P_SYMBOL.md) **§4** | `0x20` (`DTYPE_FUNCTION<<4`) iff `[sym+0x30]==3` with `+0x31 ∈ {0x54,0x55,0x56}`, or `[sym+0x30]==4` with `+0x37 & 0x400`; else `0`. `0x10b2823b`, 38 bytes, and it had **no row anywhere in the record** before 2026-08-19 |
| **U7** | What charges c2's compiler-label counter, and is the fitted `LABEL_SEED_GAP = 9` right? | [`P_LABEL.md`](P_LABEL.md) **§2, §4** | **163 charging sites, and the population is closed** — the allocator `FUN_10b97dd0`'s VA is never taken, so its **31** direct calls plus the label constructor `FUN_10b9a455`'s **132** are all of them, and `0x10b97de5` is the sole `inc`. **But 42 of the 163 sit on loop back edges**, so `charge(TU)` is a sum over c2's object population, not a per-construct table. And **the 9 is not a constant**: `7 + 2·[/Og] + 1·[/GF ∧ a string pooled in the data phase]`, measured over 22 cells — `/Od` reads **7** and is one of the 18 graded lanes (latent, not live) |

---

## 4. Coverage — with denominators, and the parts that are small

**Report the fraction, not the impression.** Targets were frozen in
[`PREREG.md`](PREREG.md) §3 before any of this was generated.

| # | denominator | measured | target | verdict |
|---|---|---|---|---|
| **C1a** | `c2.dll` addresses cited anywhere under `docs/` — **1 126 at base `071d2d47`, 1 129 at this tip** (the lane's own amendments added three) | **1 129 of 1 129 = 100%** have a row in `ADDR.tsv` | ≥ 95% | **HIT** |
| **C1b** | the same 1 129 | **907 = 80.3%** resolve to a containing function with a size (over all 1 199 rows including the label-only ones: 972 = 81.1%) | ≥ 70% | **HIT** |
| **C2** | **4 916** functions Ghidra found in the image | **631 distinct functions** are named by at least one row = **12.8%** | *no target, by design* | reported |
| **C3** | **6** prioritized subsystems | **6 of 6** have a page; entry counts 21 / 24 / 33 / 32 / 16 / 19 | 6 of 6, ≥ 8 entries each | **HIT** |
| **C4** | **19** `WB_*_FINDINGS.md` documents | **18** are back-linked from at least one row (86 distinct docs in total) | ≥ 15 | **HIT** |

`ADDR.tsv` has **1 199 rows** — the 1 129 cited addresses plus 70 that carry a
hand label and have never been cited in prose. **376 rows carry a subsystem
page**, i.e. **823 do not**: most cited addresses are outside the six
prioritized subsystems (board `#3260`).

> **The index is self-referential and the count drifts.** `ADDR.tsv` counts
> citations across `docs/`, and these pages are under `docs/` — writing prose
> that names an address adds a row. `ref/` itself is excluded from the scan, but
> `C2_MAP.md`, `BOARD.md` and the rung are not. **Re-run `build_ref.py` after
> writing, and quote the count with the tip it was taken at**, exactly as
> `STATUS.md` requires for every other generated figure.

Per-page coverage against its own band (Ghidra function entries in the span):

| page | entries | band | denominator |
|---|---:|---|---:|
| [`P_COFF.md`](P_COFF.md) | 21 | `0x10b281af`–`0x10b2b0dd` | 120 |
| [`P_SECTION.md`](P_SECTION.md) | 24 | `p2symtab.c` + `emit.cpp` anchors | 137 |
| [`P_REGALLOC.md`](P_REGALLOC.md) | 18 + 15 data | `0x10b2c21d`–`0x10b3219f` | 70 |
| [`P_DAG.md`](P_DAG.md) | 24 + 8 tables | `dag.c` + the scheduler band | 61 |
| [`P_INLINE.md`](P_INLINE.md) | 16 | `0x10b5b86d`–`0x10b62b00` | 93 |
| [`P_ENCODE.md`](P_ENCODE.md) | 71 addresses / 79 arms | `0x10bf96d0`–`0x10bfae2a` (`code.c`) | 14 |
| [`P_LABEL.md`](P_LABEL.md) | 31/31 allocator sites + 132 located | `0x10b97dd0` / `0x10b9a455` and their 163 call sites, image-wide | 163 |

> **2026-08-18, lane `w-sizebracket`** — `P_INLINE.md` gained §2.1a/§2.1b/§2.1c
> and a ⛔ correction box, and `ADDR.tsv` was regenerated: **1,209 rows, 1,141
> cited in `docs/`**, resolved-to-a-containing-function **981/1,209 = 81.1 %**,
> at tip `dd127956`. The +12 cited addresses are that lane's own amendments —
> §4's self-referential drift note, firing exactly as written. The C1a/C1b
> targets are unaffected and are not restated: **the row above is
> `w-c2map2`'s measurement and stays as it was taken.**
| [`P_EH.md`](P_EH.md) | 19 | `0x10be04e7`–`0x10be3800` | 47 |

**Deliberately NOT covered, stated so absence does not read as coverage:**
`globopt.c`, `globlopt.c`, `lur.c` (the loop rewriter — 15 115 lines), all four
`pogo*` files and the 104-import `pgodb100.dll` client (dead on this workload),
`dbg.cpp` / `.debug$S`, `ltcg.c`, `inlnasm.c`, `ptinl.c`, `ssa_seh.c`, the
`.ex` opcode semantics, and the instruction-selection tables — three lanes have
already read those and `WB_SELECT_RECONCILED.md` is their join; this reference
links to it and does not re-read it.

**87.2% of the image's functions have no entry here.** That is the honest
number and it is not going to be small after one lane.

### 4.1 `FUNCS.tsv` — the complement, at a 100 % denominator (2026-08-19, `w-c2map3`)

The paragraph above is the reason this file exists. `ADDR.tsv`'s coverage is
**bounded by prose** — `build_ref.py`'s seed set is literally
`addrs = set(cites) | set(labels)` — so an arriving lane holding an address
nobody has written about gets **silence**, not even *"this is in `globopt.c`'s
gap, 3 callers, references string X"*.

[`FUNCS.tsv`](FUNCS.tsv), generated by `docs/whitebox/scripts/build_funcs.py`,
has **one row per function in the image: 4,916 Ghidra functions + the 1 verified
Ghidra-missed entry of §6.2 = 4,917, i.e. 100 %.** Columns: TU and its
confidence, `subsys`/`page`, `cover ∈ {paged, labelled, cited, none}` **rolled
up from the address to the containing function**, the label text, degree, `hop`,
`nstr`/`strings`, `nimp`/`imports`.

Measured at this tip:

| | value | note |
|---|---:|---|
| rows | **4,917** | the denominator is the image, which is the whole point |
| `cover` paged / labelled / cited / **none** | 122 / 164 / 243 / **4,388** | **529 = 10.8 % of functions** are covered at all |
| TU attribution that is a **fact** (`in-anchor`) | **1,435 = 29.2 %** | the other 70.6 % are `gap` hypotheses; 104 sit below the first anchor |
| a **strong hook** (a string literal or an import) | **520 = 10.6 %** | string 331, import 271 |
| within 6 call hops of a covered function | 4,803 = 97.7 % | **and this is the column that does not work — see below** |

> **The `hop` column was built as a triage instrument and it does not triage.**
> 2,196 functions (44.7 %) sit at hop 2 and 1,427 at hop 1, so *"two calls from
> the register allocator"* describes almost half the image. Only the extremes
> carry signal: `0`, and the **134** functions at `-`/`5`/`6+` that are genuinely
> isolated. Kept and labelled rather than dropped, because the next lane will
> otherwise re-derive the same ordering and believe it. **Fifth entry in this
> repo's *"ranking instruments measure themselves"* pattern, and the lane's own
> prereg invalidation rule #3 is what caught it.**

Note the two indices count different things and **both counts are right**:
`ADDR.tsv`'s C2 row says *632 distinct functions are **named by a row***, which
includes data addresses whose `func` cell is the datum itself; `FUNCS.tsv` says
**529 code functions are covered**, rolled up per function. Quote the one you
mean.

---

## 5. Retrieval — the full body and the raw bytes

The flat export is machine-local, is **not committed** (bulk decompiled
third-party C is not an in-tree artifact), and regenerates in minutes per
[`../C2_MAP_METHOD.md`](../C2_MAP_METHOD.md) §3–4. It is referenced through
`$C2RS_GHIDRA_EXPORT`, defaulting to `~/ghidra-projects/export/c2`.

```sh
E="${C2RS_GHIDRA_EXPORT:-$HOME/ghidra-projects/export/c2}"
A=10b2e7f8

grep -P "^$A\t"  docs/whitebox/ref/ADDR.tsv     # everything already known
grep -P "^$A\t"  "$E/calls.tsv"                 # callees
grep -P "\t$A\t" "$E/calls.tsv"                 # callers
awk "/^\/\/ ===== FUNC $A /{p=1} p; /^\/\/ ===== FUNC /&&p&&!/$A/{exit}" "$E/decomp_all.c"
grep -n "^$A" "$E/objdump_intel.asm"            # raw bytes, at the correct VA
```

**Never open the Ghidra project** — it is a single-writer database and
concurrent access corrupts it. Everything downstream greps the flat export.

Regenerate `ADDR.tsv` after adding a page or a label:

```sh
python3 docs/whitebox/scripts/build_ref.py
```

A page **owns** an address when that address is the **first cell of one of the
page's table rows**. Authoring a page is what puts an address on it; the index
never guesses.

---

## 6. Two calibration notes from building this

### 6.1 The address regex is wrong in three ways, and this lane shipped one of them

The obvious pattern for "an address in prose" is `\b10[bc][0-9a-f]{5}\b`. **It
is wrong in three directions and this lane's own PREREG denominator (1 079) was
built with a variant of it:**

* `\b` does **not** fire between `x` and `1`, so `\b10[bc]…` **misses every
  `0x10b9b8e9`** — the most common form in the record;
* dropping the anchors entirely **matches substrings** inside the sha256 and
  inside byte dumps, which is where 1 079 came from;
* the record writes `FUN_10b8303c`, `DAT_10c2e234`, `LAB_10c1bfe2` as often as
  bare addresses, and `_` is a word character, so **both** `\b` and
  `[^0-9A-Za-z_]` anchoring miss all of those.

The corrected count is **1 126** at base, and the difference is not cosmetic: the
`FUN_`-prefixed form alone was 120 addresses, including `FUN_10b8303c`, the
symbol-list driver that is the whole of open question R6.

**The transferable point:** a denominator produced by a one-line grep is an
instrument, and an instrument that has not been tested is not evidence. This is
the fourth entry in this repo's *"ranking instruments measure themselves"*
pattern.

### 6.2 `0x10b7f022` is a real function Ghidra never created — and `C2_MAP.md` §3E's location is wrong

`ADDR.tsv`'s first pass left **17 addresses in the emit walk unresolved**, all
between `0x10b7f022` and `0x10b7f1e5`. Reading `objdump_intel.asm` there:

```
10b7f021:  c3                       ret
10b7f022:  56                       push   esi           <- a function entry
...
10b7f362:  e9 bb fc ff ff           jmp    0x10b7f022    <- reached by TAIL JUMP
```

`0x10b7f022` is a real entry — a `push esi` immediately after a `ret`, and the
target of a tail jump from `FUN_10b7f1ff`. Ghidra's auto-analysis created **no
function** there, so the range belongs to nothing in `functions.tsv` and
`decomp_all.c` has no body for it.

> **Consequence:** `C2_MAP.md` §3E says the emit walk loop at `0x10b7f15f` is
> *"inside `FUN_10b7f1ff`"*. **It is not.** `0x10b7f15f` is *below* that
> function's entry address; the loop is inside the tail-jump target
> `0x10b7f022`, and **grepping the export for `FUN_10b7f1ff`'s body will not
> show it.** The finding itself — the emit predicate — is unaffected; only its
> location is. The correction is carried in `build_ref.py`'s `GHIDRA_MISSED`
> table and amended beside §3E, not written over it.

This is worth generalising: **"Ghidra found 4 916 functions" is a statement
about Ghidra**, and a tail-called routine is exactly the shape it misses. Any
claim of the form *"address A is inside function F"* should be checked against
`F`'s entry and size, which `ADDR.tsv` now prints.

---

## 7. Provenance and status of this directory

* **Tier 1 — no white-box debt.** The 53 original source file names
  (`coff.c`, `coffemit.c`, …) are plain `strings` output of c2's C1001 path, on
  the same footing as the obj and the `/FAsc` listing.
  [`../C2_MAP.md`](../C2_MAP.md) §3A.
* **Tier 2 — white-box.** **Every address on every page here.**
* **This lane adopted nothing into `crates/`** and added no `DISCLOSURE.md`
  row. `crates/`, `fixtures/` and `scripts/` are byte-identical at both ends of
  it.
* **Nothing here is evidence about the port's correctness.** A white-box
  reading is a hypothesis; the real `c2` under wibo plus a byte-exact obj
  compare remains the sole judge.
