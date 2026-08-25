# WB_RELSITE — PRE-REGISTRATION (the IL-opcode → relation-code SITE, missed by two lanes)

    Tag:       w-relsite
    Date:      2026-08-25
    Kind:      CHARACTERIZATION lane (`../rungs/README.md` § "Lane kinds")
    Base:      a8593651b
    Branch:    wt-w-relsite
    Board:     #3546-#3550 reserved
    Fixtures:  none · Census: +0 · predicted reach: 0 · zero `crates/` bytes
    Workload:  15a64d92f197
    Image:     compilers/X360/16.00.11886.00/c2.dll, sha256
               c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
               — VERIFIED by this lane against `C2_MAP_METHOD.md` §0 before any
               disassembly was taken.

**Frozen before any disassembly. Committed as this lane's FIRST commit.**
Nothing below is edited afterwards; the score in `WB_RELSITE_FINDINGS.md` §N
grades against this text as committed.

---

## 0. The assignment, restated so it can be scored

`w-relread` §9 item 1 (= `WB_RELREAD_FINDINGS.md` §10 item 1) = `w-c7`'s prereg
**W2**: **where, in c2, does an IL opcode become a relation code?**

Known going in, and none of it is mine:

* the relation codes are **read**, from the 19-entry name array at `0x10c38690`
  and six consumer sites — `0 ILLEGAL, 1 EQ, 2 NE, 3 LT, 4 GT, 5 LE, 6 GE,
  7 ULT, 8 UGT, 9 ULE, 10 UGE, 11 SO, 12 NSO, 13 S, 14 NS, 15 VALL, 16 NVALL,
  17 VNONE, 18 NVNONE` (`#3518`);
* the port's IL model reads `0x1F => Eq, 0x20 => Ne, 0x21 => Le, 0x22 => Lt,
  0x23 => Ge, 0x24 => Gt` (`crates/c2-il/src/func/mod.rs:1411-1416`),
  byte-graded `[O]`;
* so the map is `1F→1, 20→2, 21→5, 22→3, 23→6, 24→4` — a **permutation**, and
  `relation code = IL opcode − 0x1E` (`w-c7` §2's own heading) is **FALSE**
  except on `EQ`/`NE`. It is **not** a surviving claim; it is what `w-c7`'s
  circular constraint 4 was built to produce (`#3518`);
* `w-relread` eliminated the **contiguous-byte-table** form image-wide: the
  forward pattern `01 02 05 03 06 04` and the inverse `1f 20 22 24 21 23` both
  have **0 hits** over every byte offset of the 1 347 072-byte image.

**The deliverable is a LOCATION** — a VA, the mapping it performs, read out of
instructions. Recovering the *value* again is not the deliverable; `w-c7`
scored a MISS for exactly that substitution and its recovered value was also
wrong.

---

## 1. WHAT ORIENTATION ALREADY FOUND — declared here so it is NOT scored as a blind prediction

**Stated up front or the score is a lie.** Before writing this file I read
`CLAUDE.md`, `docs/STATUS.md`, `docs/rungs/README.md` § "Lane kinds",
`docs/rungs/2026-08-24-w-relread.md`, `docs/whitebox/WB_RELREAD_FINDINGS.md`,
`C2_MAP_METHOD.md` §0, grepped `docs/BOARD.md` for `#3490`/`#2207`/`#2102`/
`#423`/`#3517` (oldest last, per the brief), and — because the brief says
*"check `docs/whitebox/` for an existing artifact covering your subject"* —
grepped `docs/whitebox/ref/P_ILRECORD.md`.

**That grep returned a named route, and it is the single most important thing
in this file:**

> `ref/P_ILRECORD.md` §arm table, **arm 7**: `10bc38a1` · opcodes `1f`…`24` (6)
> · class 00 · 13 B · ROUTE/DEFER · **→ `0x10bbffbb`. "All six relational
> operators share one arm and no discriminator is passed — the callee re-reads
> the opcode through `ecx+4`."**

So a landed lane (`w-read-r5`, boards `#3415`–`#3421`) had **already read the
arm that receives all six relational IL opcodes and named its callee**, months
of lane-time before two subsequent lanes searched for this site and missed it.
I did not find that by cleverness — the brief told me to check
`docs/whitebox/` for an existing artifact, and `P_ILRECORD.md` is one.

**This is declared as ORIENTATION, not as a finding and not as a prediction.**
Everything below is registered *given* that route. What is **not** yet known
and is what this lane must read: whether the mapping happens inside
`FUN_10bbffbb`, in what form, and where the resulting code is stored.

**A prior-art claim is itself a claim and will be checked, not assumed**: I
will verify arm 7's VA, its opcode set and its callee **from raw image bytes**
before citing `P_ILRECORD.md` as right (M1 below). `w-relread`'s D5 is the
precedent — a confidently-phrased prior artifact was wrong at code 13.

---

## 2. The registered predictions

`p` is my credence. Every row names what would falsify it.

### S — the site

| # | prediction | p | falsified by |
|---|---|---|---|
| **S1** | Arm 7 verifies from raw bytes: a ~13-byte arm at `0x10bc38a1` whose only call is to `0x10bbffbb` | 0.80 | a different callee, a different length, or a discriminator being passed |
| **S2** | The opcode→code conversion happens **inside `FUN_10bbffbb`**, not one level deeper | 0.55 | `FUN_10bbffbb` defers again with the opcode still unconverted |
| **S3** | I name the site with a VA and the instruction that materialises the code | 0.70 | I do not — decline per §4 |
| **S4** | The form is a **compare/`sub`+`je` chain with per-opcode literal `mov`s** (or a 6-way jump table of literal-loading arms), **not** a contiguous byte table — `w-relread` eliminated the table form image-wide | 0.65 | a table lookup indexed by the opcode, or arithmetic on the opcode |
| **S5** | The conversion is **not** arithmetic on the opcode (no `sub eax,0x1e` producing the code) | 0.85 | any `sub`/`lea`/`add` of `0x1e`/`0x1f` feeding the stored code |
| **S6** | The literals I read will be exactly `1,2,5,3,6,4` for opcodes `1F,20,21,22,23,24` — i.e. the code **CONFIRMS** the permutation and **REFUTES** `code = opcode − 0x1E` from the code rather than from the tables | 0.75 | any other literal set |
| **S7** | The code is stored into the node field **`+0x34`** (what `FUN_10c1a908` reads at `0x10c1a91c`, unmasked) | 0.40 | stored at `+0xa`, `+0x8`, or into a different record entirely |
| **S8** | `FUN_10bbffbb` is **< 1 500 bytes** | 0.60 | it is larger |
| **S9** | The same function also mints the node opcode (an IR opcode ≥ `0x2af`, per `WB_ILRECORD_FINDINGS.md` §0) for the compare node | 0.50 | no `0x2xx` literal in the body |

### T — is there more than one site?

| # | prediction | p | falsified by |
|---|---|---|---|
| **T1** | There is **exactly one** opcode→code site for the six value-position relations | 0.55 | a second, e.g. a separate guard/branch-position path |
| **T2** | The **guard/branch** position (`if (a<b)`) reaches the *same* site — `#423`'s guard-position grid and the value position share the conversion | 0.45 | a distinct converter for the branch form |

### U — the namespace check (this is the one that can invalidate the whole read)

| # | prediction | p | falsified by |
|---|---|---|---|
| **U1** | The byte `FUN_10bc2d7a` dispatches on **is** the same byte the port's `c2-il` reader calls the IL opcode — i.e. `0x1F..0x24` in the dispatch are the port's `Eq..Gt` | 0.80 | the dispatch's opcode space and the container's opcode space differ |

**U1 is registered because `w-c7`'s error was a cross-namespace coincidence
read as confirmation** (`#3518`). If U1 fails, S6 is meaningless whatever
literals I read, and the honest outcome is a decline naming the namespace gap.

### M — methodology controls, registered as controls and not as predictions

| # | control | registered expectation |
|---|---|---|
| **M1** | **Decode from raw image bytes.** No claim rests on `ADDR.tsv`, `FUNCS.tsv`, `data.tsv`, `P_ILRECORD.md` or any prior lane's prose. Prior artifacts are used for NAVIGATION only and every navigational fact is re-derived | p = **0.25** that at least one prior artifact I navigate by disagrees with the raw decode |
| **M2** | **Two independent disassembly sources** where a claim is load-bearing (objdump + Ghidra export, or objdump at two boundary-aligned start offsets per `w-relread`'s D4) | p = 0.30 that they disagree somewhere |
| **M3** | **Watch any fence I write refuse.** Any script I add verifies the image sha256 before parsing, and is shown refusing a truncated image, a same-size bit-flipped image, and an unreadable path | p = **0.25** that my first fence is wrong (`w-relread` D1 fired on exactly this) |
| **M4** | **End in a confirmation probe against real `c2.dll` under wibo.** `[R]` means *the instructions were read correctly*, not *this is what c2 does* — the `.bss` bump rule is the standing counterexample. The cheap probe here is **U1**: capture the IL for a TU containing all six relations and check the opcode bytes present against the read | p = 0.70 the probe confirms; a MISS here outranks every `[R]` above |
| **M5** | **Publish the denominator with any null.** Any "0 occurrences" claim names the population searched | — |
| **M6** | **Refuse at least one name** rather than guess between the read and the check | p = 0.6 that I refuse at least one |
| **M7** | **Count how many of a document's constraints the ALTERNATIVE also satisfies** before calling anything over-determined | — |

### Marker convention (used on every claim, never blurred)

`[R]` read out of the pinned image by this lane · `[O]` measured over objs,
cited to the lane that measured it · `[I]` an inference joining them, **not a
finding**.

---

## 3. What this lane will NOT do

* **No probe grid.** Read-before-probe is standing doctrine and is the whole
  point of the assignment (`WHITEBOX_LEVERAGE_2026-08-21.md`). `#423`'s 36-cell
  grid was retired as a dispatch question precisely because reading is cheaper.
  The only compilation this lane may run is (a) `scripts/gate.sh` as a
  non-regression check and (b) the single M4 confirmation capture.
* **No `crates/` byte.** `git diff --stat <base>..HEAD -- crates fixtures c2host`
  must be empty at the tip, and is checked.
* **No adoption**, therefore no `DISCLOSURE.md` row is expected to be due. If
  any address below is baked into `crates/` by a later lane, that lane owes the
  row.
* **No renaming of the enum, the three tables, or `#423`.** Those are settled
  or bounded elsewhere and this lane does not relitigate them.

---

## 4. The DECLINE FLOOR, fixed before reading

`w-read-r8` was the riskiest read in the read plan for exactly this reason and
declining cleanly was accepted in advance. This lane's floor:

1. **Depth bound: three levels of callee from arm 7.** `0x10bc38a1` →
   `0x10bbffbb` → one callee → one more. If the opcode is still unconverted at
   depth 3, **I stop and decline.**
2. **Byte bound: 8 000 bytes of body read.** If the search has read more than
   that without finding a literal keyed on the opcode, **I stop and decline.**
3. **A decline names what was eliminated, with denominators** — which
   functions were read, how many bytes, which forms are excluded — and is
   scored a `declined` outcome, not a `FAILED` one. `FAILED` is reserved for
   producing none of the deliverable, and an eliminated search space with its
   denominator IS part of the deliverable.
4. **A guess is never substituted for the read.** If I can see *where* the
   conversion must be but cannot read it, I say so and mark it `[I]`.

**Outcome word** will be exactly one of `converted | declined | instrument |
built | FAILED`. Given `Fixtures: none` and `Census: +0`, `converted` is
impossible by construction; the live options are `instrument` (address-cited
findings landed under prereg), `declined` (floor §4 hit), or `FAILED`.

---

## 5. Gate

`sh scripts/gate.sh --jobs 4 --require-graded` must be green before the lane
closes. **It is quoted as NON-REGRESSION ONLY** — a docs-only lane structurally
cannot move the gate's result, and `w-relread` §7.1's correction is adopted
here in advance: *"docs-only" is a statement about the diff, never about the
CPU.* The box is shared with three peer lanes, one of which (`w-hygiene`) is
running a timing experiment; the gate is scheduled around it and the check is
`ps`, not optimism.
