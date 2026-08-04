# w-refs — pre-registration. Frozen BEFORE the first measurement against truth.

    Lane:      w-refs, 2026-08-04, worktree `wt-w-refs` off master `73e5831`
    Question:  does replacing w-emit's 26-token `.ex` operand PROXY with the real
               `.gl` PER-SYMBOL REFERENCE LIST close w-roots' recall gap without
               costing precision?
    Status:    PREREG. No number below has been measured against truth `E`.
               Scored in the findings file, hit or miss, misses first.

---

## 0. Why this lane exists, in one paragraph

w-roots showed the emit ROOT set is *readable*: bit `0x20` of the `.gl` flag word
at `sym+0x4c` is a perfectly sound seed oracle (`Seed ⊆ E`, **14 662 / 14 662**,
zero exceptions, 850 TUs) and its closure over w-emit's **26**-token `.ex` proxy
reaches **74.200 %** of every emitted name at precision **0.99991**
(F1 **0.85186**, +64.4 pp over emit-everything). It then declined — correctly —
to swap in the `.gl` reference list, because the motivation to reach for it
arrived *after* the registered prediction failed. **This lane is that swap,
pre-registered.** It changes exactly one thing: the edge relation. `Seed`, the
truth reader, the population, the universe `U` and the closure operator are
w-roots' as landed.

---

## 1. What I read out of the binary BEFORE writing this file

Every load-bearing instruction re-read with `objdump` on
`compilers/X360/16.00.11886.00/c2.dll` (image base `0x10b00000`, `.text` VA
`0x1000` @ file `0x400`). `work/w-refs/dis.sh` reproduces each.

### 1a. The READER — `10b9bf99` … `10b9c007`

The reference list is the tail of the **tag-`0x0E` arm** of the shared `.gl`
record handler at `10b9bdcf`:

| VA | bytes | meaning |
|---|---|---|
| `10b9bf46` | `83 7d 88 0e  0f 85 ce 00 00 00` | `cmp [ebp-0x78],0xe; jne 10b9c01e` — **everything below is tag-`0x0E` only** |
| `10b9bf99` | `f7 46 4c 00 10 00 00  74 67` | `test [esi+0x4c],0x1000; je no-list` — the list is gated on the **same flag word** whose `0x20` bit is the seed |
| `10b9bfa9` | `83 3d 70 d0 c6 10 00` | `cmp ds:0x10c6d070,0` — count is `i32c` when set, `i16c`+`movzx` when clear |
| `10b9bfce` | `e8 48 39 08 00` | `call varU` — **the token** |
| `10b9bfd6` | `e8 cb 39 08 00  0f b7 d8` | `call i16c; movzx ebx,ax` — **the use count** |
| `10b9bfde` | `66 85 db  74 20` | `test bx,bx; je skip-the-alloc` — **a zero-use entry is parsed and then DROPPED. It is not an edge.** |
| `10b9bff1` | `89 48 04  66 89 58 08` | `node[4]=token; node[8]=count` |
| `10b9c003` | `83 7d 90 00  75 c2` | loop |

### 1b. The WALKER — `10b276e4`, which is why this list is *the* emit relation

    Mark(sym, edx):
      if (sym[0x4c] & 0x20) return;            // already marked -> stop
      if (ds:0x10c462c4 && edx == 0) return;
      sym[0x4c] |= 0x20;                       // MARK — the seed bit itself
      if (!ds:0x10c3cf68) return;
      ecx = sym[0x80]; if (!ecx) return;       // the reference list
      for (node = ecx[0xc]; node; node = node[0]):
          tgt = node[4][4]
          if (tgt[0x37] & 0x400) continue;     // storage-class nibble 0xa: SKIP
          if (tgt[0x4c] & 0x20) continue;      // already marked
          Mark(tgt, edi)

`p2/main.c`'s walk loop at `10b7f16b` then compiles every symbol whose `0x20`
survived. **So c2's emit set is the least fixpoint of `flags4c |= 0x20` over
exactly this list** — which is precisely the model w-roots built with a proxy in
place of the list. `+0x37 & 0x400` is set at `10b9be44`
(`and ecx,0xfffffe9f / or ecx,0x480`) for storage-class nibble `0xa`, the nibble
being bits 0..3 of the `GetByte` at `10b9be0e`.

Seven further call sites of `Mark` exist (`10b27731`, `10b28ca3`, `10b3389b`,
`10b98be8`, `10b98c08`, `10b98c7f`, `10b9aa26`). **They are additional ROOT
sources, not additional edges.** `10b28ca3` in particular marks a function
whenever a *data* node resolves to it (`test [edi+0x37],0x200000` — the tag-`0x0E`
marker set at `10b9bf50`). This lane does **not** model any of them; see §6.

### 1c. The structural fact that shapes my predictions — established pre-freeze,
### from the IL, without touching truth

**The reference list is carried by tag-`0x0E` (function) records only.** The
`cmp [ebp-0x78],0xe / jne` at `10b9bf46` puts the `+0x54` anchor, the `+0x4c`
flag word *and* the list behind the same tag test, so tags `0x04` and `0x10`
never reach it. Data symbols therefore have no on-disk reference list, which
contradicts the hypothesis `PHASE7_VALIDATION.md` §7 carries from `glgraph.py`'s
docstring — *"for data symbols too, so a static table of function pointers links
to everything whose address it takes, and a vftable links to its slots"* — a
hypothesis §7 itself labels *"from a docstring, not a measured fact"*.

Checked against the data as well as the disassembly, on `src/App.cpp`
(`.gl` = 1 512 566 bytes, 12 151 name runs):

| name | token | occurrences in the whole `.gl` | in `.ex` |
|---|---|---:|---:|
| `?EaseBackInOut@@YAMMMM@Z` — w-roots' canonical address-taken case | `e5 cf 01 00` | **1** (its own header) | 1 |
| `??_GMessage@@UAAPAXI@Z` — w-roots' canonical vtable-slot case | `d8 f9 01 00` | 2 | 1 |
| `??_7Message@@6B@` — the vftable itself | `d6 f9 01 00` | 8 | 13 |

`??_7Message@@6B@` is referenced by every `Message` constructor and by
`~Message` — the vftable is a **target** of the list. Its own record is a
tag-`0x02` record with no list, so **there is no vftable → slot edge in `.gl`**,
and `?EaseBackInOut`'s token does not appear anywhere in `.gl` except in its own
header, so **there is no `gEaseFuncs[]` → function edge in `.gl` either**.

**I am registering predictions that follow from this**, and they are the ones a
measurement can most cheaply refute.

### 1d. Instrument validation run before freezing — no truth read

Two settings are not fixed by the disassembly alone (they are run-time globals):
`ds:0x10c6d070` (count width) and, inherited from w-roots, the unconditional
owner-`varU`. Both are solved against a **known-answer gate that involves no c2
output**: a decoded list must end **exactly** at the next record's header
(`<tag> <varU token> <0x00|0x26> <name>`).

| TU | records | list bit set | terminus OK (`i32c` count) | terminus OK (`i16c`) | discriminating records |
|---|---:|---:|---:|---:|---:|
| `src/App.cpp` | 6 208 | 6 208 | **6 207** | 6 206 | 1 |
| `src/lazer/game/Game.cpp` | 6 784 | 6 784 | **6 783** | 6 782 | 1 |
| `src/system/os/HolmesUtl.cpp` | 137 | 137 | **136** | 136 | 0 |

`ds:0x10c6d070` is therefore taken as **nonzero → `i32c` count**, fixed here,
before any truth is read. A "discriminating record" is one whose count field is
the `0x80` escape byte, i.e. one on which the two readings differ at all — there
are few, and the wide reading wins every one of them.

**And the published known answer reproduces.** `PHASE7_VALIDATION.md` §7 prints
the reference list of `?HolmesXboxPath@@YA?AVString@@PBD0@Z` — nine names, found
by `glgraph.py`'s *over-approximating payload scan*, at dc3 rev `fbf097a5`. My
from-the-disassembly decode, at dc3 rev `940d07dc`, returns **exactly those nine,
in that order, with use counts** (`?c_str@FixedString@@QBAPBDXZ` ×2,
`??1String@@UAA@XZ` ×4, the four `String` ctors, `??4String@@QAAAAV0@PBD@Z`,
`?FileQualifiedFilename@@YAXAAVString@@PBD@Z`, `DmMapDevkitDrive`,
`??$MakeString@…`) plus one token the symbol index does not resolve. **Zero
missing, zero extra.** This control could have gone red — a wrong pair layout or
a wrong count width would have produced garbage names — and it did not.

---

## 2. Definitions, frozen

    Seed(t)  = w-roots' seed, byte-identical: { name of a gate-clean tag-0x0e
               `.gl` record : (flags4c & 0x20) and not (flags4c & 0x02) }
    E(t)     = truth: COMDAT leaders of every IMAGE_SCN_CNT_CODE section
               (w-emit's reader, unchanged)
    U(t)     = names of gate-clean tag-0x0e records (w-roots' `record.scan`)
    R26      = w-roots' TIGHT `.ex` proxy: exb[p-1] == 0x26 and exb[p-2] != 0x67,
               STRICT `.gl`-named owners only — THE INCUMBENT EDGE RELATION
    RGL      = THE NEW EDGE RELATION: for each gate-clean tag-0x0e record with
               flags4c & 0x1000, the decoded (token, use-count) list of
               10b9bf99..10b9c007; an entry with use count 0 is NOT an edge
               (10b9bfde); tokens resolved through `il.gl_symbol_index`;
               targets restricted to U; targets whose storage-class nibble is
               0xa are skipped (10b276e4 / 10b9be44)
    RU       = RGL ∪ R26
    P_X      = closure(Seed, X) ∩ U, for X in {R26, RGL, RU}

Population: the **850** TUs of `work/emitpred/magnitude/truthlist.txt`, the same
IL (`work/w-emit/il`) and the same truth (`work/w-emit/truth`) w-emit captured
and w-roots scored. **The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md`
is honoured and NOT spent** (§5a).

**`Seed`, the truth reader, `U`, `R26` and the closure operator are NOT to be
tuned.** Exactly one thing changes in this lane: the edge relation.

## 3. Frozen predictions — points, with intervals, against the NAMED INCUMBENT

**The incumbent is w-roots as landed, not a bare threshold**: `|P_26|` =
**129 430**, precision **0.99991**, recall **0.74200**, micro-F1 **0.85186**,
per-TU exact sets **132 / 850**. Second incumbent, kept for scale:
**emit-everything** (the port's behaviour today) = precision 0.11577, recall 1.0,
F1 **0.20752**.

| # | quantity | point | interval | what a miss means |
|---|---|---:|---|---|
| **N1** | **RGL closure RECALL — the headline** — `\|E ∩ P_RGL\| / \|E\|` | **0.76** | [0.70, 0.85] | **≤ 0.74200 ⇒ the real reference list is no better than the proxy**; > 0.90 ⇒ the gap IS closed and my §1c structural read is wrong |
| **N2** | RGL closure PRECISION — `\|P_RGL ∩ E\| / \|P_RGL\|` | **0.9995** | [0.9950, 1.0000] | < 0.995 ⇒ the swap costs precision, and the answer to the lane's question is "no" on the second half regardless of N1 |
| **N3** | RGL closure micro-F1 | **0.862** | [0.820, 0.910] | must beat the incumbent **0.85186 by ≥ 2.0 pp** (i.e. ≥ **0.87186**) to count as an IMPROVEMENT; within ±2 pp is a **WASH**; below 0.83186 is a **REGRESSION** |
| **N4** | union `RU` recall | **0.79** | [0.72, 0.90] | reported; ≥ 0.90 ⇒ the two relations are complementary and the gap is closed by taking both |
| **N5** | edge agreement — fraction of `R26` edges that are also `RGL` edges | **0.90** | [0.70, 1.00] | < 0.70 ⇒ the two instruments are measuring different things and no comparison of their closures is interpretable on its own |
| **N6** | per-TU exact sets — `P_RGL == E` over 850 TUs | **0.17** | [0.05, 0.45] | reported, not gated (incumbent 0.15529) |
| **N7** | **residual shape — vtable-slot share** of `E ∖ P_RGL` (virtual member ∪ `??_G`/`??_E`), w-roots' classifier unchanged | **0.37** | [0.25, 0.50] | **< 0.25 ⇒ the reference list DID carry vtable-slot edges and §1c is refuted** |
| **N8** | **residual shape — free/file-scope share** of `E ∖ P_RGL` | **0.44** | [0.32, 0.56] | **< 0.32 ⇒ the reference list DID carry address-taken edges and §1c is refuted** |
| **N9** | **the missing edges are not in `.gl` at all** — among `E ∖ P_RGL` names whose `.gl` token is 4 bytes wide, the fraction whose token byte string occurs **exactly once** in the whole `.gl` | **0.70** | [0.40, 0.95] | reported, not gated; this is §1c's claim generalised off one TU |
| **N10** | `\|P_RGL\|` — the predicted set size | **135 000** | [110 000, 175 000] | reported; a value near `\|U\|` = 1 506 586 would mean the list is not a sparse relation |

**Bias declaration.** §1c gives me structural grounds to expect the swap to fail
to close the gap, and I was briefed that "declining on measurement is a good
result here", which is a bias **towards** reporting failure. I have therefore
registered **N1 above the incumbent** (0.76 > 0.74200) and **N3 as an
improvement** (0.862 > 0.85186), so that "no change at all" costs me two misses,
and I have registered N7/N8/N9 tightly enough that a reference list which *does*
carry the two named classes shows up as three simultaneous misses. **The single
outcome I most expect to be wrong about is N1 landing below 0.742** — if it
does, the proxy beats the real relation, which would be worth more than either
of the two answers the brief anticipates.

**What I predict does NOT move:** TU match stays **8**. `census/gate
disagreement` stays **0**. No `crates/` change, no fixture, no widening —
`PortC2` keeps returning `NotImplemented` outside its class.

## 4. Known-answer controls — registered pass marks

| # | control | pass mark |
|---|---|---|
| **KA-A** | **reproduce the incumbent exactly** on the same 850 TUs with the same readers: `\|E\|` 174 417, `\|U\|` 1 506 586, `\|Seed\|` 14 662, `\|P_26\|` 129 430, F1 0.85186 | **exact** on all five. This is the whole basis of the comparison; a drift here voids N1–N6 |
| **KA-B** | **the TERMINUS gate** — a decoded list must end exactly at the next record's `<tag><varU><0x00\|0x26><name>` header | ≥ **0.98** of list-bearing records over 850 TUs, **and the count of discriminating records (count field = the `0x80` escape) must be printed and be > 0.** Failing records are counted, never decoded anyway |
| **KA-C** | **the published witness** — `?HolmesXboxPath@@YA?AVString@@PBD0@Z` reproduces `PHASE7_VALIDATION.md` §7's nine names | **9/9, zero extra.** Already run pre-freeze (§1d) and re-asserted by the committed script |
| **KA-D** | **MUTATION against the SOLE JUDGE** — pick 5 functions that are in the obj, are not seeded, and are reached from `Seed` by **exactly one** `RGL` edge; set that edge's use-count byte to `00` (an `i16c` small positive is one byte, so the mutation is **byte-length preserving**), replay the mutated `.gl` through the real `c2.dll` under wibo, compare objs | **≥ 3/5** lose exactly that COMDAT. This tests the SEMANTICS (`test bx,bx / je` at `10b9bfde`), not the layout. **A survivor is informative and will be reported with which of the two explanations applies** (the edge is not load-bearing / another root reaches it), never buried |
| **KA-E** | incumbent gate on the unmodified tree (`cargo test --workspace --release` with the **target count**, `gate.sh`, `selftest`, `cargo build --release`, `c2rs gap`) | every incumbent reproduced; **compare the FAILED count, never the passed count** |
| **KA-F** | dc3-decomp HEAD recorded **before and after** the run; wibo `--version` recorded | a mid-run corpus move is a **hard error**, not a footnote |
| **KA-G** | **the positive check: this run GRADED the swap** — the number of names on which `P_RGL` and `P_26` DISAGREE, printed | **> 0**, printed as a count. If the two closures are identical the run graded nothing about the swap and **must say so as a failure**, not report N1 as a tie |

**Would KA-D go red if the claim were false in the most likely way?** The most
likely false claim is *"the use-count field is not what I think it is"* — in
which case zeroing it either changes nothing (COMDAT survives) or desyncs the
record (c2 errors out or emits a wildly different obj). Both are visible, and
both are scored as misses. KA-D is also the only control here that consults the
sole judge; KA-B and KA-C are self-consistency and prior-publication checks and
cannot substitute for it.

## 5. Decline clauses — priced, and each says what I will conclude

1. **KA-B < 0.95** ⇒ **the decode is not trustworthy. Publish no precision,
   recall or F1 at all.** Report only where the chain broke and on what
   population. A number from an ungated decoder is worse than no number.
2. **N1 ≤ 0.74200 (the incumbent recall)** ⇒ **declare that the real `.gl`
   reference list does NOT close the recall gap, say so in the first line, and
   STOP.** Specifically, and this is the load-bearing clause: **I will not go
   looking for a second edge channel, a different record kind, a data-initializer
   stream, an extra `Mark` call site, or any root source beyond `Seed`, in order
   to make the number move.** I may *characterise* the residual (as w-roots did)
   and I may *name* the missing channel in §6 so that absence never reads as
   success — but naming it is where this lane stops.
3. **0.74200 < N1 < 0.90** ⇒ **"IMPROVED BUT NOT CLOSED".** Report the delta
   against the incumbent, in both directions, and stop. Clause 2's no-chasing
   rule applies identically.
4. **No instrument tuning to improve a number.** The count width is fixed to
   `i32c` by §1d's terminus gate *before* any truth is read; the zero-use drop
   and the storage-class-`0xa` skip are transcribed from named instructions
   (`10b9bfde`, `10b276e4`/`10b9be44`) and are not knobs. `Seed`, `U`, `R26`, the
   truth reader and the closure operator are w-roots'/w-emit's as landed. Any
   variant I report goes **beside** the headline with its own label, never in
   place of it.
5. **Nothing ships under `crates/`.** Even if every number hits, one lane's
   decode is not grounds for widening the gate. Implementation is a separate lane
   with its own prereg.
6. **A refuted §1c is reported as a refutation of ME, first line, before any
   number that survives.** If N7 or N8 lands below its interval, the structural
   read in §1c is wrong and that is the headline, not a footnote.

### 5a. The one-shot Part-1 gate — I pre-register that I will NOT spend it

w-emitpred's 21-TU held-out quarantine is unspent. **I register, before seeing
any number, that I will not spend it**, for the same reason w-roots gave and one
more:

> A **decode has no free parameters.** The list layout is five field reads taken
> from named instructions; the one run-time boolean (`ds:0x10c6d070`) was fixed
> against a terminus gate that involves no c2 output at all, and is recorded in
> §1d with the discriminating-record count. There is nothing here for a held-out
> population to catch. **And this lane is, by construction, comparative** — its
> whole content is `RGL` against `R26` on the *same* 850 TUs, which 21 held-out
> TUs cannot improve on.

**The condition that would reverse this, registered now:** if I choose *anything*
— an edge kind, a refcount threshold, a skip rule, an exclusion — by looking at
truth `E`, then the result is fitted, I will **say so explicitly**, and the gate
becomes **owed** by whoever ships it. **I will not spend it myself in either
case.** If a later lane believes it has the finished emit model and wants to
spend it, that is a decision to bring back to the coordinator, not to take here.

## 6. What this lane will NOT measure — named now, so absence never reads as success

1. **The other seven `Mark` call sites** (`10b27731`, `10b28ca3`, `10b3389b`,
   `10b98be8`, `10b98c08`, `10b98c7f`, `10b9aa26`). These are **additional
   ROOTS**, and `10b28ca3` — "a data node resolved to a function record, mark it"
   — is the shape the address-taken class needs. Not decoded, not modelled.
2. **The data-initializer stream itself.** w-emit's capture kept only `gl` and
   `ex` of the `_CL_*` quintet; `in`, `db` and `sy` were not retained, so a
   channel living there is not merely unmeasured but **uncapturable from the
   cached corpus**. Re-capturing 850 TUs is a separate lane.
3. **Tag-`0x02`/`0x04`/`0x10` records.** They carry no list (§1c). Whether they
   carry references some *other* way is not tested.
4. **Order.** A right set in the wrong order is still a mismatch.
5. **`-optref`** (`FUN_10b27b7f`), the only path that clears `0x20`. Absent from
   the workload.
6. **The 21 quarantined TUs.** Untouched (§5a).
7. **Whether `RGL` is the relation for anything other than emit.** The walker at
   `10b276e4` is one consumer; others are not traced.
