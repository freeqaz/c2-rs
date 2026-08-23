# WB_SUB4F — FINDINGS and self-grade for read R9 (the `0x4F` sub-record switch)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Image
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified before the first address was read. **No `DISCLOSURE.md` row is
> owed**: that file is the ledger of findings *adopted into `crates/`*, and
> this lane adopted nothing and changed no `crates/` byte.

**Lane** `w-read-r9` · **kind** characterization · **Fixtures** none ·
**Census** +0 · **reach** 0, registered and delivered.
**Prereg** [`WB_SUB4F_PREREG.md`](WB_SUB4F_PREREG.md), committed **first** at
`fe9a08f39`. **Spec** [`ref/P_SUB4F.md`](ref/P_SUB4F.md). **Instruments**
[`scripts/dump_sub4f.py`](scripts/dump_sub4f.py),
[`scripts/sub4f_probe.py`](scripts/sub4f_probe.py). Board **#3442**–**#3444**.

---

## 0. The headline

**`0x10b26268` is not a width table. It is a table of format-string
pointers**, and `FUN_10b9761e` is a format-string interpreter. The "~14-arm
switch" is real and the count is exactly right — **but its arms are
field-type codes, not sub-opcodes.** The sub-opcode domain is **64**.

Priced 1–2 days, delivered inside it, **completeness achieved**: all 64
sub-opcodes, all 14 arms, all 5 scalar readers, both list siblings.

**The port's transcribed width is CONTRADICTED, and the contradiction is
live.** `4F 01 <byte>` is three bytes only while the source line is below
128; at line ≥ 128 the record is **seven** bytes. Two `crates/` sites bake the
three. **Reported, not fixed** — §5.

---

## 1. Self-grade against the prereg

Nine predictions, registered with confidences before any byte of the target
was read. **Misses are reported as misses.**

| # | prediction | conf | verdict |
|---|---|---:|---|
| **P1a** | the true distinct-arm count is **not** 14 | 0.60 | **MISS** — it is exactly 14 (13 codes + default) |
| **P1b** | sub-opcodes `M` > distinct arms `N` | 0.65 | **HIT** — `M` = 64, `N` = 14 |
| **P2** | the `4F 01` payload is a **varint**, not a fixed byte | 0.75 | **HIT** — VI32 (`0x10c1f9e9`): 1 byte, or `0x80` + 4 |
| **P3** | the switch selector is a **descriptor field**, not the sub-opcode | 0.60 | **HIT, amended** — it is a byte *dereferenced through* the descriptor pointer and iterated, which is stronger than "a field" |
| **P4a** | the entry is `{u32,u32}` with at least one **pointer** | 0.55 | **HIT** — dword 0 is a `const char *` |
| **P4b** | at least one field is a **small integer** usable as a width/class code | 0.70 | **MISS** — dword 1 is 0 or 1 and is **read nowhere in the image**; no width is stored anywhere |
| **P5** | `M ≥ 7` and all seven corpus-witnessed sub-opcodes are admitted | 0.85 | **HIT** — 7/7 admitted, none reaches a refusal arm |
| **P6** | the unexplained `47` gets an explanation | 0.45 | **HIT** — `4F 12` is **2 bytes**; `47` is the next record's opcode |
| **P7** | `0x10b97807` lies in the `0x16` arm | 0.80 | **HIT on the address — and it produced a larger finding**, §4 |
| **P8** | `FUN_10b9761e` and `0x10bbe561` are different phases, neither calling the other | 0.70 | **HIT** — disjoint; and they read the *same field* at two different widths |
| **P9** | table-derived widths parse real IL exactly | 0.70 | **HIT on the graded clause** (6,840/6,840 + 10/10); one registered clause reported **UNGRADED**, §3.1 |

**8 HIT · 2 MISS · 1 HIT-with-amendment · 1 clause UNGRADED.**

The two misses are worth more than the hits. **P1a** was a bet that the
coordinator's "~14 arms" was wrong the way R5's "189 arms" was wrong; it was
not — *the number was right and the thing it counted was wrong*, which no
prediction in the file anticipated. **P4b** assumed a width has to be stored
somewhere; it is not stored anywhere, because a width is not a property of a
sub-opcode at all — it is a sum over a code string, three of whose codes are
data-dependent.

---

## 2. Corrections this lane makes to the record

Eight, of which three were made **before the target was read** (prereg §0)
and are reproduced here for one place to look.

| # | document | said | actually |
|---|---|---|---|
| 1 | `DECISIONS_2026-08-22.md:275`, and the dispatch brief | R5 handed R9 its entry points, *"arms 48/49"* | **arm 32** → `0x10bbe561`. Arms 48/49 are **R6's**. `docs/rungs/2026-08-23-w-read-r5.md:195-198` is a compound sentence naming two reads; it was mis-transcribed into the decision record |
| 2 | `READ_PLAN:178`, `WHITEBOX_LEVERAGE:90`, the brief | *"the **one** transcribed width in the port"* | **at least five readers carrying two inconsistent widths** — 2 fixed-byte, 3 varint, plus `ehscope.rs:126`. `IL_STMT_GRAMMAR.md:236-240` already called the fixed one a live bug |
| 3 | — | `FUN_10b9761e` is unread ground | **R3 already read part of it** (`P_LABEL.md:51`, `0x10b97807`). Registered as P7 |
| 4 | `DISCLOSURE.md:89` (`W-MID-4`) | *"read off an 8-byte-stride descriptor table … Named so a future lane knows **where the record's widths live**"* | the table holds **format-string pointers**; no width is stored in it, or anywhere else. The *addresses* in that row are all correct |
| 5 | `ref/ADDR.tsv:93` | `0x10b26268` — `data`, size **4**, confidence `unknown` | **512 bytes**, 64 entries, stride 8, parsed |
| 6 | `WB_MIDDLE_INTERFACES.md:201` | `` `4F 01 <byte>` — source-line record `` | under-general: `<byte>` only for lines < 128 |
| 7 | `ref/P_LABEL.md:51` | *"seed install, **IL directive** `0x16`"* | `0x16` is a **field-type code**, not a directive — and **no descriptor selects it**, §4 |
| 8 | `READ_PLAN:103` | *"the `0x4F` sub-record's **~14-arm switch**"* | right count, wrong level. 14 field types over 64 sub-opcodes |

Corrections 4–8 are **amended beside, never rewritten in place**
(`ref/README.md:72+`). No peer lane's page is touched.

---

## 3. The confirmation probe

`READ_PLAN` §5.3: `[R]` is a hypothesis. Both probes were specified in the
prereg **against six named failure modes** before either was built.

### Probe A — the twin grid (aimed at FM4, the vacuous-green trap)

Ten sources with a **byte-identical body**, differing only in a leading
`#line L`, at `L ∈ {1, 100, 127, 128, 129, 200, 16383, 16384, 100000,
1000000}`. Graded on the grid's own internal consistency: cell `L=1` fixes
the marker-offset set `K = {0..6}`; every other cell must decode to exactly
`{L + k}`.

**10/10 cells PASS.** The decisive cell is **`#line 127`**, which contains
**both widths in one file** — a 3-byte record for line 127 and 7-byte records
for 128–133. `L = 1000000` round-trips, which a 16-bit field cannot hold, so
the gate `DAT_10c2eb4c` is proved non-zero in this configuration: `[O]`.

**The probe can fail, and this was verified rather than asserted.** Re-grading
**the identical captures** under the port's fixed-one-byte rule:

```
1        PASS      100      PASS
127      FAIL: got [127,128,128,128,128,128,128]
128..1000000  FAIL — every line >= 128 decodes as the constant 128
  => 2/10
```

**FM4 is thereby made visible rather than argued about.** Cells `L=1` and
`L=100` pass under **both** rules — and every fixture in this project's own
corpus looks like those two. That is exactly the shape board #2668 recorded
and `w-read-r3` found: a constant green only because its control could not
exercise it.

### Probe B — the corpus (aimed at FM5, and at being surprised)

All **386** tracked fixtures under `fixtures/cpp`, compiled by the real
toolchain under wibo. **386/386 captured. `4F 01` records checked: 6,840.
Width violations: 0.** Sub-opcodes observed: `01, 02, 0a, 0b, 10, 11, 12, 1f,
20, 33`.

### 3.1 Probe B's histogram clause went RED, and it is reported UNGRADED

The prereg registered as a red condition: *"any sub-opcode observed in the
corpus that the pinned table sends to the ICE arm, or that lies outside the
table's 64 entries."* **On the first run it fired**, reporting `0x53`, `0x54`,
`0x4d`, `0x4f`, a negative index, and `0x0a`/`0x26`.

Per the brief's standing rule — *if a result looks like it refutes you,
suspect the instrument before the claim* — the contexts were dumped:

```
4f 02 20 00 4f 01 4f 53 53 26 ...
                  ^^ line 79 ^^ the scan reads THIS payload byte as a record
```

`0x4F` is also a perfectly ordinary **payload value** (source line 79) and the
low byte of 2-byte operand tokens. Span exclusion — consuming each record
whose width the table makes computable — removed every out-of-table candidate
and `0x26`. **67 `0x0a` candidates survive and are NOT records.**

The decisive evidence is a **positive fact about the toolchain, not an
argument about the scan**: `0x10b33526` ends in `int3`, so a real sub-record
carrying an unhandled format code **kills c2** — and **386/386 fixtures
compiled and captured successfully**. Therefore none of the flagged
candidates is a record, and the observation *confirms* the table read.

**The clause is reported UNGRADED, not relaxed.** The registered rule assumed
a soundness the superset scan does not have, and no span exclusion can fix
that without the full `.ex` grammar, which nobody has. Silently rewriting a
preregistered rule to obtain a green is the failure this project's prereg
ladder exists to prevent. The **width** clause — which the scan *can* decide,
because it checks a record located by its own decoded value — is the graded
one and it passed 6,840/6,840.

---

## 4. The uncomfortable finding: `P_LABEL.md:51`'s arm is unreachable

`ref/P_LABEL.md:51` records `0x10b97807` — inside `FUN_10b9761e` — as the
label-seed install for *"IL directive `0x16`"*, `[R]`. **P7 HITs**: the
address is in the field-type-`0x16` arm.

But of the 13 handled field-type codes, **exactly one is selected by no
descriptor in the table, and it is `0x16`**:

```
handled by the cascade (13):  0b 0c 0d 0e 14 15 16 17 1a 1d 1e 6c 73
used by some descriptor (29): 02 07 08 09 0a 0b 0c 0d 0e 14 15 17 18 19 1a
                              1d 1e 41 43 46 49 4b 4d 62 64 6c 6f 71 73
handled but NEVER selected:   16
```

So **`0x10b97807` cannot be reached by any `0x4F` sub-record.** R3 read the
instructions correctly; the arm is simply **not on the path** — the `.bss`
bump failure mode (`ref/README.md:54-60`), which is precisely what `[R]` is
defined not to exclude. Asserted by `dump_sub4f.py --selftest` so it cannot
rot.

**Bounded, deliberately:** this does **not** touch `LABEL_SEED_GAP`.
`w-seedgap` measured its coefficients black box from a 22-cell obj grid
reading `u32_le(.gl[7..11])` (`DISCLOSURE.md` row `W-SEEDGAP-1`), so no
shipped number depends on this arm. What is removed is a **mechanism story**,
not a value. `P_LABEL.md` is amended beside, never rewritten, and this lane
edits no claim of R3's.

---

## 5. DEFECT REPORT — the port's transcribed width is wrong, and is NOT fixed here

**Reported for a follow-up lane. This lane changed zero `crates/` bytes.**
`w-s1bc` is the only lane in `crates/` this wave, and `w-read-r3` set the
precedent by declining to fix a shipped defect under a docs-only fence.

| site | code | correct for | wrong for |
|---|---|---|---|
| `crates/c2-reference/tests/middle_interfaces.rs:716-724` | `p += 3` | line < 128 | **line ≥ 128 → 7 bytes** |
| `crates/c2-il/src/codec.rs:1128-1145` | `let nn = *body.get(p+2)?; … 3` | line < 128 | same |

`crates/c2-il/src/func/readers.rs:409-416`,
`shapes/control_flow.rs:534-542` and `body/expr.rs:1300-1306` already read a
varint and are **correct**. `crates/c2-il/src/func/ehscope.rs:126` steps a
`4F 01` run inside a `while` and should be re-checked by whoever takes this.

**Severity, priced honestly and two-sided.** Every fixture in `fixtures/cpp`
sits below line 128, so the defect is **latent, not live** on the current
corpus — which is why 6,840/6,840 records pass and why no gate has ever gone
red on it. It becomes live on any real TU whose functions live past line 127,
i.e. essentially every one, which is exactly what `IL_STMT_GRAMMAR.md:236-240`
says. `codec.rs`'s reader is on a shipping path; `middle_interfaces.rs`'s is a
test.

**What this read adds** over the existing suspicion is the *reason*: the width
is VI32, `0x10c1f9e9`, one byte or `0x80` plus four, selected by field-type
code `0x6c` under gate `DAT_10c2eb4c` — and a grid that makes the boundary
fail on demand.

---

## 6. What this lane did NOT do

- **No `crates/` byte.** The branch's whole diff against master is `docs/`.
- **No `DISCLOSURE.md` row** — nothing adopted. R1, R3 and R5 each owed none.
- **No obj graded, no emit rule, no refusal predicate, no fixture.** Nothing
  here is `[O] port`.
- **No peer lane's page edited.** `w-read-r6`/`r7`/`r8` write
  `docs/whitebox/`; this lane created only its own four files and appended
  three board rows.
- **Semantics beyond field types.** This is a grammar. What `0x1f`'s VI32 or
  `0x33`'s list *mean* is not read.
- **Field-type `0x0b`'s full width** — `0x10b97d47` consumes further stream;
  read to depth 2 and **DEFER**red.
- **The default arm's behaviour on invalid IL.** Prereg §6.4 registered this
  as out of scope: it needs feeding c2 malformed input. What *is* now known is
  that the refusal is **fatal** (`int3`), not a skip.

---

## 7. What these controls remain structurally incapable of catching

Registered in prereg §6 before any result, and **none of it was discharged**:

1. **Most of the domain is gradeable by no probe this lane can build.** Board
   #3096 measured that no body in 870 TUs reaches a `0x4F` sub-opcode other
   than `01`/`12`. This lane's 386 fixtures **establish seven** — exactly
   `IL_STMT_GRAMMAR` §12.6's set `{01,02,11,12,1f,20,33}`, each occurring
   hundreds to thousands of times. The scan reports three further candidates
   (`0x0a` ×67, `0x0b` ×4, `0x10` ×2) which are **two to three orders of
   magnitude rarer and are not distinguishable from payload bytes** (§3.1), so
   they are not counted as witnessed. **At least 54 of the 64 slots were never
   exercised**, and their rows are `[R]` and stay `[R]` — including **every
   one of the 17 ICE rows**, which is the half of the deliverable no probe
   built from valid IL can ever reach.
2. **One front end, one version.** Both probes measure c2's reader against
   c1xx 16.00.11886.00's writer; they agree by construction wherever c1xx
   never goes. The 17 ICE sub-opcodes and the 17 unhandled format codes are
   evidence *about the table*, not about what some other producer emits.
3. **Exact consumption is necessary, not sufficient.** Probe B is a strong
   falsifier and a weak verifier. 6,840/6,840 is not proof.
4. **A fifth failure mode, unnamed until the run** — and R2 hit this too. The
   prereg's six named modes did not include *"the scan's own definition of a
   record is unsound"*, which is what §3.1 turned out to be. It was caught
   only because the contexts were dumped rather than the rule relaxed.
