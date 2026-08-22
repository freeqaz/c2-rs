# P_LABEL — the compiler-label counter `DAT_10c2edd0` and its 163 charging sites

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`../DISCLOSURE.md`](../DISCLOSURE.md).
> Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
> **verified before the first address on this page was read**, on both the repo
> path and `~/ghidra-projects/bin/c2dll` (the flat export's input).
> Marks are [`README.md`](README.md) §2's: `[R]` read, `[O]` obj-confirmed,
> `[I]` inferred. **An unmarked claim is a defect.**

**Lane** `w-read-r3`, read **R3** of
[`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3, funded by the
owner 2026-08-22 ([`../../DECISIONS_2026-08-22.md`](../../DECISIONS_2026-08-22.md)
decision 1). Prereg [`../WB_LABELCHARGE_PREREG.md`](../WB_LABELCHARGE_PREREG.md)
(committed first). Grade [`../WB_LABELCHARGE_FINDINGS.md`](../WB_LABELCHARGE_FINDINGS.md).
Board **#3387**–**#3390**.

**This page gives the CHARGE, not the ORDER.** A charge rule without an order
rule still cannot place a label; the other half is **R8** (block emission
order), and `READ_PLAN` §3 says so in the row itself. Nothing here tells you
*which* block a `$M` lands on.

---

## 0. The one-paragraph answer

c2 numbers its compiler labels out of a single 32-bit TU-global,
`DAT_10c2edd0`, seeded from the IL because `c1xx` and c2 share one id space.
Exactly **one instruction** increments it — `inc DWORD PTR ds:0x10c2edd0` at
**`0x10b97de5`**, inside the 28-byte allocator `FUN_10b97dd0` — and that
allocator's address is **never taken** anywhere in the image, so its **31
direct call sites are the entire population of charges**, plus the **132**
sites of the generic label constructor `FUN_10b9a455`, which is itself one of
the 31. **The site population is closed. The CHARGE is not**: **42 of those
163 sites sit on loop back edges**, so a TU's total charge is a
data-dependent sum over whatever population the loop walks, not a per-construct
constant. That distinction is the whole content of this page, and it is why
four lanes measured the subject wrong and why `LABEL_SEED_GAP` turns out not to
be a constant either.

---

## 1. The counter

| thing | VA / value | mark |
|---|---|---|
| **the counter** | **`DAT_10c2edd0`**, one 32-bit TU-global | `[R]` |
| references to it **in the whole image** | **7**, every one in `.text` | `[R]` |
| — the only **increment** | **`0x10b97de5`** `ff 05 d0 ed c2 10` | `[R]` |
| — seed install, IL directive `0x16` | **`0x10b97807`**, in `FUN_10b9761e` | `[R]` |
| — seed install, per-TU header | **`0x10b97ca1`**, in `FUN_10b97a22` | `[R]` |
| — reads | `0x10b8b5cd` (the crossing check), `0x10b97c90`, `0x10b97dd0`, `0x10b97de0` | `[R]` |
| the **downward** end of the same id space | `DAT_10c2ed40`, written at `0x10b8b5c7` in `FUN_10b8b561`, with the crossing check `if (DAT_10c2ed40 <= DAT_10c2edd0) fatal` | `[R]` |

Reproduce the reference count without Ghidra:

```sh
python3 docs/whitebox/scripts/dump_label_sites.py \
        compilers/X360/16.00.11886.00/c2.dll --refs 10c2edd0
```

> **THE WRITE SET PARTITIONS INTO TWO KINDS AND THERE IS NO THIRD** `[R]`.
> Two **assignments from outside** (the seed) and **one `+1`**. No
> `add [mem], k` for `k > 1`, no decrement, no per-function reset, no
> conditional double-bump. Whatever c2's charge is, it is a **count of
> executions of one instruction**.

### 1.1 The allocator — 28 bytes, one path

```text
10b97dd0: 83 3d d0 ed c2 10 00   cmp    DWORD PTR ds:0x10c2edd0,0x0
10b97dd7: 75 07                  jne    0x10b97de0
10b97dd9: 6a 37                  push   0x37
10b97ddb: e8 d6 70 08 00         call   0x10c1eeb6      ; internal error 0x37, no return
10b97de0: a1 d0 ed c2 10         mov    eax,ds:0x10c2edd0
10b97de5: ff 05 d0 ed c2 10      inc    DWORD PTR ds:0x10c2edd0
10b97deb: c3                     ret
```

`[R]` **Every call that returns charges exactly +1.** The only branch leaves
through a non-returning internal-error call, so there is no path that yields a
number without incrementing and none that increments twice. The guard is also
the proof that the counter is **seeded, not zeroed**: asking for a number
before the seed is installed is ICE `0x37`.

### 1.2 The seed

`[R]` `FUN_10b97a22` (805 B) is the IL-header reader. It validates the front
end's version stamp, then:

```c
_DAT_10c2eaa0 = FUN_10c1fb8b();                    /* a u32 out of the IL stream */
if (_DAT_10c2eaa0 < DAT_10c2edd0) _DAT_10c2eaa0 = DAT_10c2edd0;
DAT_10c2edd0 = _DAT_10c2eaa0;                      /* = max(IL value, current)  */
```

**No constant is added.** `[O]` The black-box form of the same fact is
`OBJ_GY_SHAPES.md` §3.5's `B = u32_le(.gl[7..11]) + 9`, and §4 below shows
where the 9 comes from and that it is **not 9 everywhere**.

> `[R]` **c1xx and c2 share one id space, and that is the whole reason the
> four wrong measurements were wrong.** The front end numbers the labels and
> symbols *it* creates, writes those ids into the IL, and hands c2 the next
> free value. c2's IL reader takes `sym[+0x28]` straight from the stream
> (`FUN_10c1f91b`, no bump — `../WB_LABEL_FINDINGS.md` §1.3), so **an IL-named
> label costs nothing**, and c2 allocates upward only for what **it** invents.
> A whole-TU displacement between two source texts therefore measures
> `Δseed + Δcharge`; see §6.

---

## 2. Closure — what "closed by construction" is true of, and what it is not

`READ_PLAN` §3 row R3 asserts the mechanism is *"closed by construction — one
increment instruction"*. Three separate claims hide in that sentence. Two hold
and one does not.

| claim | verdict | evidence |
|---|---|---|
| **(a) one increment instruction** | **TRUE** `[R]` | §1; 7 references, 3 writes, 1 of them arithmetic |
| **(b) the charging call sites are an enumerable, complete population** | **TRUE** `[R]` | the allocator's VA `0x10b97dd0` occurs **0 times** as a 4-byte absolute anywhere in the image, so there is no function pointer to it in any table, vtable or callback slot; a direct `call` encodes a *relative* displacement and never the target, so the 31 `E8` sites are all of them. Independently derived twice — an `E8 rel32` scan over raw `.text` from the pinned image, and the Ghidra export's `xrefs.tsv` — **agreeing exactly at 31 and 132** |
| **(c) therefore the CHARGE is a closed-form constant per construct** | **FALSE** `[R]` | **42 of the 163 sites are loop-resident** — 3 of the 31 (§3.2) and 39 of the 132 (§7). A loop-resident site charges once per element of whatever the loop walks, so the charge is `Σ over a data-dependent population`. An enumeration of *sites* is not an enumeration of *charges* |

> **THE CORRECTION, PLAINLY.** The site table is closed and finite. The charge
> is a **sum over c2's own object population**, and that population is what a
> port would have to reproduce — which is a much larger obligation than
> "replace the fitted `+9`/`+3`" suggests. `READ_PLAN` §3 row R3's *"closed by
> construction … replacing the fitted `+9`/`+3`"* is **half right**: it
> replaces the `+3` and it **refutes** the `+9` (§4), but it does not turn the
> per-function charge into a constant, and nothing in this read does.

The loop-residency test is mechanical and reported as such `[R]`: a site is
loop-resident iff some jump `J` inside the containing function jumps backward
across it (`target(J) <= site < addr(J)`, both inside the function). It is a
*conservative* test in the safe direction — it can call a site loop-resident
when the back edge cannot actually re-reach it, never the reverse.

---

## 3. The 31 allocator sites

Denominator **31/31 read** to *(caller, guard, object kind)*.
`tu` is `ref/FUNCS.tsv`'s attribution (tier 2, `mech` confidence — weaker than
`[R]`, see `FUNCS.tsv`'s own header).

| site | caller | tu | fires | what takes the number |
|---|---|---|---|---|
| `0x10b283c0` | `FUN_10b283b0` | *(gap)* | once/call | **COMDAT spin-off**: a child section built by `FUN_10be74cf` from a base section; stores `".data"` into the child when `base[+0x4d]==1` `[R]` |
| `0x10b28734` | `FUN_10b28586` | `coff.c` | once/TU, guarded `DAT_10c45d6c == 0` | the **`.drectve`** section — `FUN_10be7473(0x10b01bc4 = `"`.drectve`"`, "DRECTVE", id, 10)`, created as the obj is opened `[R]` |
| `0x10b5903a` | `FUN_10b5902e` | `globregs.c` | once/TU, guarded `DAT_10c2e460 == 0` | the **`.rtc$IMZ`** section (name string read at `0x10b023e0`), group `"CONST"`, kind 4 `[R]` |
| `0x10b59068` | `FUN_10b5902e` | `globregs.c` | same guard, same call | the **`.rtc$TMZ`** section (`0x10b023d4`) `[R]` |
| `0x10b590bc` | `FUN_10b59091` | `globregs.c` | once/call | a named **`"CONST"` COMDAT** for an RTC datum, plus a symbol and a tuple `[R]` |
| **`0x10b5cee1`** | `FUN_10b5ceb5` | `hash.c` | **LOOP** — nested, `0x400` buckets × chain | **the bulk symbol-numbering pass.** `sym[+0x28] = FUN_10b97dd0()` for every symbol in the table **except** kind 1 with the 3-bit linkage field `(sym[+0x37] & 0xe00000) == 0x600000`. One charge **per symbol** `[R]` |
| `0x10b72c35` | `FUN_10b72c0a` | `list.c` | once/call | the **`.cil$<suffix>`** section that carries a verbatim IL file into the obj `[R]` |
| `0x10b72d39` | `FUN_10b72d14` | `list.c` | once/call | the **`.cil$fg`** section (name string `0x10b132f0`), group `"ILFILE"` — the flag/command-line record `[R]` |
| `0x10b803a4` | `FUN_10b8034a` | `misc.c` | once/call, guarded by a **miss** in the `DAT_10c2ed14` cache **and** a miss in the name table | a **kind-1 symbol** with `+0x31 = 0x26`, `+0x8 |= 0x2000` — an interned synthetic datum `[R]` |
| `0x10b828de` | `FUN_10b8289c` | `misc.c` | once/call | a **kind-0xe** object (`FUN_10b984c3(0xe,4,1)`), `+0x30 = 4` — the formatter's `$E` family `[R]` |
| `0x10b85739` | `FUN_10b855b9` | `misc.c` | once/TU, guarded `DAT_10c46b5c == 0` | the **arena/pool head** kind-1 symbol, `+0x37` linkage 6, `+0x4e = 5` `[R]` |
| `0x10b96978` | `FUN_10b968b0` | `optimize.c` | once/call | a **kind-0xe** object named by `sprintf_s`, `+0x37 \|= 0x200000` `[R]` |
| `0x10b96a69` | `FUN_10b968b0` | `optimize.c` | once/call, guarded `(fn[+0x20] & mask) != 0` | a **section** via `FUN_10be74cf` for that object `[R]` |
| `0x10b9a223` | `FUN_10b9a143` | `p2symtab.c` | once/call, guarded `sym[+0x30]=='\4'` inside `(sym[+0x20]&0x20) && sym[+0xc][0x4c]!=7 && !(sym[+0x20]&0x20000000)` | the **section a kind-4 symbol goes in** (`FUN_10be74cf`) `[R]` |
| `0x10b9a268` | `FUN_10b9a143` | `p2symtab.c` | the `else` arm of the same guard | the **section a non-kind-4 symbol goes in**; the section's own kind is chosen from `sym[+0x20]` bits `0x20000`/`0x40000` `[R]` |
| **`0x10b9a468`** | **`FUN_10b9a455`** | `p2symtab.c` | once/call — **and this is the 132-site constructor** | **a kind-3 (label) object**: `+0x31 = 0x20` (anonymous, caller overwrites), `+0x28 = the number`, then `+0x3f = DAT_10c2e918++` `[R]` |
| `0x10b9a4bc` | `FUN_10b9a4a7` | `p2symtab.c` | once/call | a **named kind-1 section-ish datum** (`FUN_10b984c3(1,4,1)`, `+0x37` linkage 7) — one of the callers is the `"__r12_indirect"` builder `[R]` |
| **`0x10b9a8d9`** | `FUN_10b9a897` | `p2symtab.c` | **LOOP** — the intern probe — but the charge is in the **`bucket == 0` (not found)** arm | **the name→symbol intern.** `FUN_10b8a01b(name) & 0x7f` into the 128-slot open-address cache `DAT_10c67db8`, linear probe. **A name c2 has already interned charges nothing** — this is the dedup mechanism §5 needs `[R]` |
| `0x10b9b5e1` | `FUN_10b9b5d2` | `p2symtab.c` | once/call | a **clone** of an existing symbol (`FUN_10b9853a`) — the clone gets a **fresh** number `[R]` |
| **`0x10b9b701`** | `FUN_10b9b6a4` | `p2symtab.c` | once/call | an **anonymous kind-1 symbol**, `+0x31 = 0x26`, `+0x47 = 1`. **This is the `$T` minter** — see §5.4 `[R]` |
| `0x10ba245f` | `FUN_10ba2422` | `p2symtab.c` | once/call | the **intermodule-call thunk's** symbol (a clone), `+0x37` linkage 3 `[R]` |
| `0x10ba3588` | `FUN_10ba34e8` | `p2symtab.c` | once/call, guarded `(sym[+0x20] & 0x20) != 0` | the thunk's own **`.text`** section (`0x10b165f0`), group `"CODE"` (`0x10b162f0`) `[R]` |
| **`0x10bdbb37`** | `FUN_10bdbaba` | `tuple.c` | **LOOP** over a tuple list, charge in the `piVar2[0xd] == 0` (first time) arm | a **kind-1 symbol per distinct switch/jump-table target group** `[R]` |
| `0x10be7918` | `FUN_10be78a8` | `emit.cpp` | once/call, **`else` arm of** `param_1 == *(int*)(DAT_10c472e8+0x2cc)` | a section id for a **non-default segment**. For the default segment c2 uses the **reserved constant `0xd`** and charges nothing `[R]` |
| `0x10be7927` | `FUN_10be78a8` | `emit.cpp` | same `else` arm | the paired id; the default-segment constant is **`0xf`** `[R]` |
| `0x10be798f` | `FUN_10be794d` | `emit.cpp` | same shape | the **`<base>$zz`** COMDAT-ordering section (kind `0x1b`); default constant **`0x19`** `[R]` |
| `0x10be79c3` | `FUN_10be794d` | `emit.cpp` | same shape | its pair; default constant **`0x1a`** `[R]` |
| `0x10be7a3d` | `FUN_10be79fa` | `emit.cpp` | same shape | the **`<base>$zy`** COMDAT-ordering section (kind `0x20`); default constant **`0x16`** `[R]` |
| `0x10be7a71` | `FUN_10be79fa` | `emit.cpp` | same shape | its pair; default constant **`0x17`** `[R]` |
| `0x10c12552` | `FUN_10c1252c` | `mdmisc.c` | once/TU | the **`.XBLD$W`** COMDAT (`0x10b200d8`) that carries the `__C2_11886` build stamp (`0x10b200cc`). Note its *first* section takes the **reserved id `0x1b`** and charges nothing; only the second is charged `[R]` |
| `0x10c21851` | `FUN_10c217fd` | `vlines.c` | once/call, guarded `sym[+0xc]==0 && sym[+0x30]=='\3' && DAT_10c701a4 && (DAT_10c701a4[+0x20]&0x20)` | a section for a `.pdata` record whose symbol has no section yet `[R]` |

### 3.1 The reserved low-id region — six sites charge **only off the default segment**

`[R]` `FUN_10be78a8`, `FUN_10be794d` and `FUN_10be79fa` all have the shape

```c
if (param_1 == *(int *)(DAT_10c472e8 + 0x2cc)) id = <constant>;   /* no charge */
else                                            id = FUN_10b97dd0();
```

with constants **`0x0d`, `0x0f`, `0x16`, `0x17`, `0x19`, `0x1a`**, and
`FUN_10c1252c` uses **`0x1b`** the same way. Those values are far below any
seed (~2,500 in every TU measured), so they are **pre-assigned ids for the
standard sections of the default segment**, not charges.

**This is the reason `.data` / `.bss` / `.rdata` globals cost the seed gap
nothing** — measured, §4.2: adding an initialized global, an uninitialized
global, a const global, a 4 KiB array, or all three at once leaves the gap at
exactly 9. `[O]`

### 3.2 The three sites that make the allocator's charge data-dependent

`0x10b5cee1` (`hash.c`) is a **nested loop over a `0x400`-bucket table and each
bucket's chain**, assigning `sym[+0x28]` to every symbol that is not
`kind 1 ∧ linkage == 3`. `0x10b9a8d9` (the intern probe) and `0x10bdbb37`
(a tuple-list walk) are the other two; the constructor adds **39** more (§7).
Together they mean the allocator's per-TU charge is `Σ over c2's symbol
population`, and that is exactly what makes
`LABEL_COUNTER.md` §1.1's `stride == minted` observation true rather than a
coincidence (§5).

---

## 4. `LABEL_SEED_GAP` is **not** a constant — the finding this read owes the port

`crates/c2-core/src/coff/label.rs:9` ships

```rust
pub const LABEL_SEED_GAP: u32 = 9;
```

fitted from 25 TUs (`OBJ_GY_SHAPES.md` §3.4/§3.5).
`../WB_LABEL_FINDINGS.md` §6 open #1 has recorded since 2026-08-09 that
**whether it moves for a TU with different section needs is UNVARIED**. This
read varied it, with `scripts/gt_label_seedgap.py`.

### 4.1 The measurement `[O]`

Same two framed functions in every cell; nothing but data or flags ahead of
them; seed read directly out of the captured `.gl`, so it cannot hide in the
answer:

```text
  gap = first($M|$T) - u32_le(.gl[7..11]) - (3 * nfuncs when /Gy)
```

| mode | base | + `const char* g = "x";` |
|---|---:|---:|
| `/Od` | **7** | **7** |
| `/Os` | **7** | **7** |
| `/Ot` | **7** | **7** |
| `/Oy` | **7** | **7** |
| `/Ob2` | **7** | **7** |
| `/Og` | **9** | **9** |
| `/Ox` | **9** | **9** |
| `/Ox /Gy` | **9** | **9** |
| `/Ox /GF` | **9** | **10** |
| **`/O1`** | **9** | **10** |
| **`/O2`** | **9** | **10** |

> `[O]` **`LABEL_SEED_GAP = 7 + 2·[the global optimizer runs] + 1·[a string
> literal is pooled in the data phase under `/GF`]`**, over 22 measured cells.
> The `+2` tracks `/Og` exactly (`/Ox`, `/O1`, `/O2` all imply it; `/Os`,
> `/Ot`, `/Oy`, `/Ob2`, `/Od` do not). The `+1` needs **both** `/GF` **and** a
> string literal reached before the first function — `const char g[] = "x"`
> (an array *copy*, no separate string object) leaves it at 9, and a string
> literal returned from a *function* is not in the data phase at all.

### 4.2 What does **not** move it `[O]`

An initialized global (`.data`), an uninitialized global (`.bss`), an
externally-visible const (`.rdata`), a 64-element initialized array, a 4 KiB
`.bss` array, three globals at once, `/Gy`, `/GS` on, `/EHsc`, `/GR`, `/Oi`,
and the workload's whole `/Oi /EHsc /GR` cluster: **all 9 at `/Ox`, all 9 at
`/O1`.** §3.1 explains why — the standard sections of the default segment take
reserved ids and charge nothing.

### 4.3 Is the port wrong today? **Latent, not live — checked, not argued** `[O]`

`/Od` is one of the **18 enumerated grade lanes** (`scripts/lanes.txt`), and at
`/Od` the true gap is **7** against the port's **9**. Two checks:

* `scripts/mode_lane.sh /Od` → `LANE-RESULT PASS graded=386 total=386
  **match=21 mismatch=0**`, and **all 21 matching TUs are data-only or empty**
  (`mvp_empty`, `wa16_bss_*`, `wsect_*`, `worder3_*`, `wnpos_*`). Not one emits
  a `$M`, so `plan_labels`' seed never reaches an obj at `/Od`.
* the `/O1` + pooled-string shape, written as a probe
  (`const char* g3 = "x";` ahead of two framed functions), returns
  **`Port=NotImplemented`** through `c2rs diff`, with the reference replay
  byte-exact.

> **So nothing the port emits today is wrong, and the constant is still wrong
> as stated.** What is live is the **licence**: `LABEL_SEED_GAP` reads as a
> compilation-independent constant, and the first rung to admit a framed
> function at `/Od`/`/Os`, or a `/O1` TU with a file-scope pointer-to-string
> initializer, inherits a silently wrong `$M` on **every function in the TU**.
> This is `LABEL_COUNTER.md` §2.1's pattern exactly, one level down.

**Which nine (or seven, or ten) — still not enumerated.** This read bounds the
candidate population to §3's once-per-TU sites (`0x10b28734` `.drectve`,
`0x10c12552` `.XBLD$W`, `0x10b85739` the arena head, `0x10b5903a`/`0x10b59068`
the two `.rtc$` sections) and shows the gap is **mode-dependent**, which
already refutes "nine fixed allocations". Attributing each unit to a site needs
a live tap on `0x10b97de5`, which this lane did not build.

---

## 5. The charge, expressed as a rule

### 5.1 The rule, and the seven measured surcharge rows

> `[R]` + `[O]` **charge(TU) = the number of objects c2 CONSTRUCTS ITSELF**,
> one per constructor call listed in §3 and §7, and **zero** for anything that
> arrives already numbered in the IL.

That single sentence re-derives `LABEL_COUNTER.md` §1.1's seven measured
surcharge rows, six of them from named addresses:

| §1.1 row | measured | the site that charges it |
|---|---:|---|
| `_fltused`, the TU's first FP function | **+1** | one new external interned at `0x10b9a8d9` (miss arm) `[I]` |
| `__savegprlr_N` / `__restgprlr_N`, each distinct N | **+2** | **two** new externals, two misses at `0x10b9a8d9` `[I]` |
| `__savefpr_M` / `__restfpr_M`, each distinct M | **+2** | same, two externals `[I]` |
| a newly pooled FP constant, each distinct `(bits,width)` | **+2** | the `.rdata` **section** object *and* the `__real@…` **symbol** `[I]` |
| a callee external **the IL names** | **0** | no constructor runs — `sym[+0x28]` comes from `FUN_10c1f91b` `[R]` |
| a helper width / constant an **earlier function already introduced** | **0** | the intern probe at `0x10b9a897` **hits**, so the charging arm is not taken `[R]` |
| a **signed `>`/`<` over two call results** | **+2** | **NOT EXPLAINED** — this row mints no symbol at all, so its two charges must be two `FUN_10b9a455` label objects on the materialisation path, and this read did not locate them |

`[O]` The two dedup rows are the ones this read can *confirm* rather than
infer, and they were re-measured here: `gpr3` strides **7**, `gpr3-dup`
(same `_29` a previous function introduced) strides **5**, `gpr3-dup-wide`
(a *different* width) strides **7**; `const1-led` **7**, `const2-led` **9**,
`const1-dup-led` **5** — every row reproducing `LABEL_COUNTER.md` §1 to the
digit, on both wibo builds present on this box.

### 5.2 `stride == minted` is NOT a law — it fails in BOTH directions `[O]`

`LABEL_COUNTER.md` §1's table holds `stride == minted` on all 28 rows, and its
own §7 retraction **R2** already narrowed that to *"minting **causes** charge;
charge is not **equal to** minting"* on the base rows. This read supplies the
direction R2 did not have — **it fails both ways** — measured on rows chosen
because they can break it:

| probe | stride | minted | what it means |
|---|---:|---:|---|
| `leaf-cmp-eq` (`a == 5`) | 1 | 1 | — |
| `leaf-branch` (`if`) | 1 | 1 | — |
| **`leaf-cmp-lt5`** (signed `a < 5`) | **3** | **1** | **charge exceeds minting by 2** |
| **`leaf-loop`** (a `for`) | **3** | **1** | **charge exceeds minting by 2** |
| **`leaf-string`** (returns `"hello"`) | **1** | **3** | **MINTING EXCEEDS CHARGE BY 2** |

> **Both directions.** `charge > minted` is c2 building **label** objects
> (§7's 132 sites) that no symbol survives to represent — and the `+2` on the
> signed relational and on a loop is exactly *two* label objects, which is
> what P3.3 predicted the mechanism to be even though this read did not
> locate the two call sites. **`minted > charge` is the interesting one**: a
> string literal creates an `.rdata` COMDAT **section symbol** and a
> `??_C@…` **symbol** and charges **nothing**, while a pooled FP constant
> creates the same *shape* of COMDAT and charges **+2**
> (`LABEL_COUNTER.md` §1.1, §2.1). So the two are built by **different
> constructors**, only one of which is on the charging list — a concrete,
> checkable asymmetry, and the place a later read should start.

### 5.3 A string literal costs 0 in a function body and **+1 in the data phase** `[O]`

The two facts above look contradictory and are not, and reconciling them is
what §4's `/GF` term is:

* **in a function body** — `leaf-string` at `/O1` (which implies `/GF`):
  stride **1**, i.e. surcharge **0**, on one, two or three literals
  (`LABEL_COUNTER.md` §2.1's own measurement);
* **in the data phase** — a file-scope `const char* g = "x";` at `/O1`,
  `/O2` or `/Ox /GF`: the seed gap goes **9 → 10**, and a second literal
  takes it to **11** (§4.1, and the narrowing grid behind it).

**Same literal, same COMDAT shape, different phase, different charge.** The
`+1` is exactly the `/GF` term in §4's rule, and it is why that term needs
*both* `/GF` and a string reached before the first function.

### 5.4 The four label kinds, and which constructor mints each `[R]`

| printed | kind (`sym[+0x30]`) | `sym[+0x31]` | number from | minted at |
|---|---|---|---|---|
| **`$M`** | 3 | `'W'` (`0x57`) | `sym[+0x28]` | **`FUN_10c21992`** (20 B, `vlines.c`) — calls `FUN_10b9a455`, sets `+0x43 \|= 1`, stamps `'W'`, attaches the label to a tuple through `FUN_10c21df3` → `FUN_10bd3824` |
| **`$T`** | 1 | anything but `'$'`/`'%'`, unnamed | `sym[+0x28]` | **`FUN_10b9b6a4`** at **`0x10b9b701`**, reached from the `.pdata` record writer `FUN_10c217fd` via `FUN_10b9c655('\6',8,4,0,'\4',0x80)` |
| **`__unwind$`** | 3 | `'T'` (`0x54`) | `sym[+0x28]` | `FUN_10be1f3f` (1 220 B, `except.c` side) — `FUN_10b9a455`, then `FUN_10bd415e`, then stamp |
| **`__catch$`** | 3 | `'V'` (`0x56`) | `sym[+0x28]` | `FUN_10be04e7` (102 B) and `FUN_10c21fd2` (247 B, `vlines.c`) |
| `__annotation$` | 3 | `'Z'` (`0x5a`) | `sym[+0x28]` | `FUN_10bbdd2b` |
| `$S` / `$SG` | 1 | `'$'` / `'%'` | `sym[+0x28]` | — |
| `$E` | 4 | — | `sym[+0x28]` | `FUN_10b8289c` at `0x10b828de` builds the kind-`0xe`/`+0x30 = 4` object |
| **`$LC`/`$LL`/`$LN`** | 3 | anything else, **unnamed** | **`sym[+0x3f]`** | any of the 132 `FUN_10b9a455` sites |

**Every `$M` costs exactly one charge, and the address is `0x10c21992`.** That
is the single most useful line on this page for the port.

---

## 6. The formatter charges nothing — 201 functions, zero calls `[R]`

`FUN_10b99dfe` (682 B) is the name formatter. Its **whole call subtree is 201
functions and contains zero calls to `FUN_10b97dd0` and zero to
`FUN_10b9a455`.** Naming a label — or never naming it — cannot move the
counter.

The switch, re-read here rather than inherited:

```c
if (sym[+0x30] == 1) {
    if (sym[+4] != 0) {                      /* named */
        if (((sym[+0x37] >> 0x15) & 7) not in {1,3}) return;   /* prints nothing */
        append "$";  if (sym[+0x4d] < 3) return;               /* then the number */
    } else if (sym[+0x31] == '$') "$S";
      else if (sym[+0x31] == '%') "$SG";
      else                        "$T";
} else if (sym[+0x30] == 3) {
    switch (sym[+0x31]) {
      case 0:   append "$";            break;
      case 'T': "__unwind$";           break;
      case 'V': "__catch$";            break;
      case 'W': "$M";                  break;
      case 'Z': "__annotation$";       break;
      default:  if (sym[+4] != 0) return;                  /* named -> nothing */
                "$L" + (sym[+0x43] & 0x10 ? "C" : sym[+0x43] & 4 ? "L" : "N")
                     + dec(sym[+0x3f])                     /* <== THE OTHER COUNTER */
                     + (param_2 && DAT_10c2e2f4 ? "@" + current function : "");
                return;
    }
} else if (sym[+0x30] == 4 && sym[+4] == 0) "$E";
else return;
FUN_10c1e739(sym[+0x28], buf, n, 10);        /* radix-10, the GLOBAL id */
```

Two things this adds to `../WB_LABEL_FINDINGS.md` §1.2 `[R]`:

* the **kind-1 named** arm is additionally gated on the **3-bit linkage field**
  `((sym[+0x37] >> 0x15) & 7) ∈ {1,3}` — the same field `P_SYMBOL.md` §3 found
  suppresses COFF records at `0x10b28bb4`. Outside it the formatter prints
  **nothing at all**;
* the `@<function>` suffix on `$L*` names is switched by the formatter's
  **second argument**, not by any symbol field.

> **`cl /FAsc` stays CLOSED as a route to the charge, and now there is a reason
> rather than a measurement.** The listing prints `$L*` from **`sym[+0x3f]`**,
> filled by the *second* counter `DAT_10c2e918` (`0x10b9a483`, reset to 1 per
> function at `0x10b7e13c` in `FUN_10b7e113`), which counts **label objects
> including the ones the IL supplied for free**. The charge is `sym[+0x28]`
> off `DAT_10c2edd0`. Two counters, two increment sites, two populations —
> which is exactly why `stride ≥ max($LN)` failed on the first row anyone
> measured (`LABEL_COUNTER.md` §7.3).

---

## 7. The 132 label-constructor sites — who invents labels, and where

`[R]` All 132 are direct `E8` calls from **86 distinct functions**; the address
`0x10b9a455` is likewise **never taken as data**. **39 of the 132 are
loop-resident.** Attributed through `ref/FUNCS.tsv` (tier 2 `mech`):

| c2 source file | sites | callers | what it says |
|---|---:|---:|---|
| `mod.c` | 19 | 9 | the largest single source of invented labels |
| `lowersmd.c` | 16 | 8 | machine-dependent lowering |
| `cgintrin.c` | 10 | 6 | intrinsic expansion |
| **`fg.c`** | **10** | **8** | **the flow graph — this is R8's own file** |
| `except.c` | 9 | 2 | and 8 of the 9 are in one function, `FUN_10be4f28`, all loop-resident |
| `tuple.c` | 8 | 6 | |
| `code.c` | 7 | 5 | the encoder's file (R2's) |
| **`lur.c`** | **7** | **6** | **loop unrolling** |
| `pogoopt.c` | 7 | 5 | |
| `ehexcept.c` | 5 | 3 | |
| `inline.c` | 5 | 4 | |
| `p2symtab.c` | 4 | 4 | |
| `ptinl.c` | 4 | 4 | |
| `regasg.c` | 4 | 2 | |
| `vlines.c` | 4 | 4 | the `$M` minter's file |
| `lower.c` | 3 | 1 | |
| `dag.c` | 2 | 2 | |
| `globopt.c` | 2 | 2 | |
| `mdmisc.c` | 2 | 1 | |
| `globregs.c`, `mdlist.c`, `optimize.c`, `pogoinline.c` | 1 each | 1 each | |
| **total** | **132** | **86** | over **23** named files |

> **THE TWO COLUMNS ARE DIFFERENT NUMBERS AND THIS LANE MISCOUNTED THEM ONCE.**
> A first pass histogrammed *callers* and reported them as *sites* — `fg.c`
> "eight sites" when it is eight callers and **ten** sites, `lur.c` "six" when
> it is six callers and **seven** sites. Corrected here and in every document
> that quoted it. Both columns are given because both are used: **callers**
> bounds how many bodies a reader must open, **sites** bounds the charge.

Busiest individual callers: `FUN_10be4f28` (`except.c`, **8 sites, all
loop-resident**), `FUN_10b8d6d7` (`mod.c`, 4), then `FUN_10c25fb4`,
`FUN_10c25bbe`, `FUN_10c0d57e` (**R6's final-expansion switch**),
`FUN_10bfefbb`, `FUN_10bf50d6`, `FUN_10be2ab8`, `FUN_10bc69f1`,
`FUN_10b8e11a`, `FUN_10b8dc2b`, `FUN_10b38326` at 3 each.

Two consequences worth stating `[I]`:

* **`lur.c` holds seven label-constructor call sites in six functions, and
  `lur.c` is 15,115 lines and
  UNREAD** (`READ_PLAN` §1's last row). That is a mechanism for
  `LABEL_COUNTER.md` §7.7 open #3 — *"the `/Ox` loop charge: four magnitudes
  (10, 3, 7, 10), no rule, and none proposed"*. At `/Ox` a loop's charge is a
  property of what the unroller did, and the unroller invents labels from six
  places nobody has read.
* **`fg.c` holds ten sites in eight functions**, and `fg.c` `0x10b36133` is where `READ_PLAN` row
  **R8** starts. R3 and R8 are the same deliverable from opposite ends, as the
  plan says; this is the concrete overlap.

---

## 8. What this page does NOT give

| # | open |
|---|---|
| 1 | **The ORDER.** Which block a `$M` lands on is R8's. A charge rule alone cannot place a label. |
| 2 | **Which units make the gap** (7 / 9 / 10). The candidate set is bounded to §3's once-per-TU sites and the mode-dependence is measured; the per-unit attribution needs a live tap on `0x10b97de5`, unbuilt. |
| 3 | **The `/Gy` `+3` per function.** Re-confirmed `[O]` here as an exact `3 × nfuncs` in every `/Gy` cell of §4.1's grid, but *what* the three are is still not read out of the binary — the same status `WB_LABEL_FINDINGS.md` §6 open #2 left it in. |
| 4 | **§5's signed-relational `+2`.** The one surcharge row this read does not explain. |
| 5 | **The 132 sites' individual guards.** They are located, attributed and loop-classified; only the handful in §5.4 are read to guard level. |
| 6 | **The `/Ox` loop charge.** §7 names a mechanism (`lur.c`); it does not model it, and this read proposes no rule. |
| 7 | **The downward pool.** `DAT_10c2ed40`'s write at `0x10b8b5c7` and the crossing check are located; the interaction between the two ends is unmeasured, exactly as in 2026-08-09. |
| 8 | **The mode-dependence's own boundary.** 22 cells over 11 flag sets and two source shapes is not the flag space. `/Oi`, `/EHsc`, `/GR`, `/GS`, `/GF`, `/Gy` were each varied; `/Zi`, `/GL`, `/arch`, PGO and `#pragma` families were not. |

---

## 9. Reproduce

```sh
sha256sum compilers/X360/16.00.11886.00/c2.dll   # must equal the digest above

# the closure argument, straight from the image — no Ghidra
python3 docs/whitebox/scripts/dump_label_sites.py \
        compilers/X360/16.00.11886.00/c2.dll --closure

# the instrument's own self-test: the banner's cells, both forms side by side
python3 scripts/gt_label_seedgap.py --selftest

# the seed-gap grid
python3 scripts/gt_label_seedgap.py
python3 scripts/gt_label_seedgap.py --mode '/O1 /GS- /c'

# the surcharge rows this page re-derives
python3 scripts/gt_label_stride.py gpr3 gpr3-dup gpr3-dup-wide \
        const1-led const2-led const1-dup-led leaf-int plain

# the licence check
cargo run -p c2-harness --bin c2rs -- diff work/w-read-r3/probe/gapstr.cpp
scripts/mode_lane.sh /Od
```
