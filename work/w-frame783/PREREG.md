# w-frame783 — PREREG

**Frozen before the first `crates/` change and before the first line of the
shipped reader.** Everything in §1 is a *finding* already measured off this
lane's own capture; everything in §3 is a *prediction* with its antecedent
spelled out. Scored in the rung's §9.

    Lane:      w-frame783, worktree branch `worktree-agent-a8d7dca60b1e3155a`
    Base:      master `5e617e8e` (the `w-selbind` merge + its STATUS regen).
               Merge-base re-derived with `git merge-base HEAD master`; the tree
               is CLEAN at freeze time.
    Workload:  dc3 `104e7df9c10acfe56ee3a87d75f0a9c85740df11`, tracked tree
               CLEAN (one untracked dir, `work/`). **Unchanged from
               `w-selbind`** — the first lane in seven that does not have to
               re-state a moved stamp (#2392).
               `work/dc3-workload/files.txt` md5 `09189d4a41713c77e14dca9af5050b58`
               (878 lines), `flags.txt` md5 `ef3b32e8ac8d3ab89a8be0a0a60e40c8`.
               **Used as they stand and never regenerated** (#2700).
    Toolchain: `compilers/X360/16.00.11886.00`, wibo `1.2.0-c2rs.1`.
    Base bin:  `c943b83634731b358767bd9008676bdf`, copied to
               `work/w-frame783/c2rs-base` before the first edit (#2409) and
               **KEPT**. Every base figure below is that binary's own scan.
    Base test count, re-derived at THIS merge-base from `STATUS.md`'s generated
               block (never from a rung): **1493 passed, 0 failed, 41 targets**;
               selftest **369 PASS / 0 FAIL**; fixture gate **150 Match / 0
               mismatch / 219 n-i of 369**.

---

## 1. `CEILING.md` §11.4, WORKED FIRST, OFF THIS LANE'S OWN CAPTURE

Target of record: `src/system/math/vec.cpp` (w-selbind's, so the two passes are
comparable), captured by this lane at the workload's own flags into
`work/w-frame783/cap/vec` — `.ex` 170,578 B, `.gl` 65,856 B.

| # | item | answer at this tree, this capture |
|---:|---|---|
| **1** | ask the BYTE judge | base scan: `vec.cpp` `fnbyte-exact` **2**, `differs` **0**, `refused` **0** |
| **2** | if every body is exact, the blocker is not codegen | **T1 fires** — unchanged from `w-selbind` |
| **3** | read the SYMBOL TABLE, not just `.text` | not re-derived; `w-selbind` §5.1 stands (9 sections, 34 symbols, 0 relocations, `_fltused` undefined-external). **This lane changes no emitter arm, so the writer's obligations are not on its path** |
| **4** | is the refusal LIST MEMBERSHIP? | no — a separator byte stops the walk |
| **5** | do not trust the key's LAYER | n/a — no key is being priced; the lane is a reader change |
| **6** | check factor A first | `factor-a 28`, `a-and-b-and-c 27`, `frontier 4` at the base scan |
| **7** | check the board | #2783, #2784, #2790, #2820–#2827, #2622/#2623, #2750/#2751/#2754/#2756, #1721 read. §4 lists what this lane declines to re-price |
| **8** | quote the GATE's number — **`gate_cause`**, and nothing else | `vec.cpp` `gate_cause` **`gl-stop-26-introduced`**, `gate_causes` `[gl-stop-26-introduced, body-out-of-class]`. **This is the item that decides this lane** — see §1.1 |
| **8b** | an instrument's population is bounded by the reader | `gl_body_record_names` runs **no walk**: it is a raw scan over `.gl` with none of the six stop clauses. So the 414 it reports is bounded by the *framing* only, and the gate's 34 is bounded by the framing **and** the walk |
| **9** | read the port's FENCES before its obligations | not reached on `vec.cpp` (the walk stops first). The three fences — `comdat::fenced_inlined_callee`, `elide`'s E, `splice`'s S7 — stay unasked and stay part of the price |

### 1.1 Item 8, and the field this lane adds to its tally

§11.4 item 8 records **five** fields used to answer *"does the gate bind this
TU"* and **four** wrong. This lane's commission rests on a **sixth**:
`gl_body_record_names`, the reader behind the published **414**.

It is *not* the gate's reader and it is not a walk at all. Side by side, from
the code as it stands at `5e617e8e`:

| reader | framing | stop clauses | what it answers |
|---|---|---|---|
| `gl_gate_record_names` → `gl::gl_bound_names` → `gl_defined_names_framed(…, codec::gl_offset_framed, InlineOrStringTable)` | GATE | **all six** (`NameTooFar`, `NameNotMangled`, `RunEndsAt26`, `DllexportLinkage`, `VariadicRecord`, `Name26Introduced`) — and any one of them empties the **whole TU** | *"what does the gate bind"* → **34** |
| `gl_body_record_names` | WIDE | **none** | *"what could any framing name"* → **414** |

**So the two published numbers differ in two things, not one**, and the
commission's *"the 380-TU gap between 34 and 414 is entirely #2783's one-byte
frame relaxation"* (#2824, carried into `CEILING.md` §13.1) is a claim about
only the first of them. §3's deciding row is whether it survives.

### 1.2 #2783 RE-DERIVED, from this lane's own bytes — the relaxation, and one byte it is missing

`codec::gl_offset_framed` (`crates/c2-il/src/codec.rs:1275`) tests seven bytes
around a `.gl` body-start field at `o`:

```text
   gl[o-7]==0x80   gl[o-5]==0x10   gl[o-4..o]==00 00 00 00   gl[o]==0x80
   80 <LE32 PREV> 00 00 | 80 <LE32 BODY-START>
   ^o-7  ^o-6…o-3  ^o-2   ^o   ^o+1…o+4
```

`PREV`'s tag is `gl[o-7]`, so `gl[o-5]` is **PREV's byte 1** and, with
`gl[o-4]==gl[o-3]==0` already required, the pinned clause is exactly
`PREV ∈ [0x1000, 0x10FF]`. `bind::emit_offset_framed` is that test with the
clause dropped, i.e. `PREV < 0x10000`. **Confirmed on this lane's own capture,
not inherited**: `??0Vector3@@QAA@MMM@Z`'s body-start 98,922 is spelled at
`.gl`+27,521 and its record's PREV is **`0x189a`**; `vec.cpp` goes **36 → 369**
framed records; the PREV byte-1 histogram over the wide set is
`0x10:36, 0x11:39, 0x12:27, 0x13:11, 0x14:27, 0x15:9, 0x16:8, 0x17:20,
0x18:140, 0x19:27, 0x1a:14, 0x1b:11` — a rising per-record number, not a tag.
(`work/w-frame783/frame783.py`.)

**Over all 876 TUs this lane could capture** (`sweep783.py`, `fpshape.py`):

| | GATE framing | WIDE framing (#2783) |
|---|---:|---:|
| framed records | **28,870** | **1,507,159** |
| framed offsets that are **not** an `.ex` `4F 1F` split point | **1** | **551** |
| TUs carrying such an offset | **1** (`src/system/utl/TempoMap.cpp`) | **406** |
| duplicate offsets | 0 | 0 |
| TUs whose records are 1:1 with the segments | **32** | **32** — 0 lost, 0 gained |
| record positions the WIDE scan does not contain but the GATE scan does | — | **0 of 876 TUs** |

Two things fall out that #2783 does not say:

* **The gate's framing is not precise either** — `TempoMap.cpp` carries one, so
  *"a value test wearing a position test's docstring"* cuts both ways.
* **The imprecision is separable by ONE more byte, cleanly, 551 for 551.**
  Every one of the 1,506,608 on-a-split framed offsets has **top byte 0**;
  every one of the 551 not-a-split offsets has top byte **≥ 2**
  (`{2:1, 4:1, 11:205, 16:29, 31:1, 51:2, 71:2, 87:98, 92:1, 95:2, 97:1,
  104:1, 106:2, 113:1, 117:203, 243:1}`). The largest real offset in the
  workload is 2,837,591 (`0x2b4c57`), so `gl[o+4] == 0` bounds the field at
  16 MB and **discards nothing real**. It is a **value** test and is named as
  one — the same honesty #2783 demands of the byte it removes.

So the shippable form of #2783 is *one byte freed and one byte pinned*:
**`PREV < 0x10000` and `BODY-START < 0x1000000`** — 1,506,608 records with
**zero** offsets that are not `.ex` split points, against the incumbent's
28,870 with one.

---

## 2. WHAT THIS LANE WILL SHIP, AND WHAT IT WILL NOT TOUCH

Design fixed before any code, following `w-decouple`'s shape (`NameFit`): the
widening goes to the **binding** and **both fences keep the incumbent walk**.

* `codec::gl_offset_framed_relaxed` — the framing above, beside the incumbent,
  which is **not modified**.
* `gl::gl_bound_names` (the gate's BINDING walk, `Bindings::selective` /
  `per_record`) reads the relaxed framing.
* `gl::gl_defined_names_with` — the FENCE ground set (`bind::defined_name_set`
  for the census, `gl::plain_external_defined_names` for the gate's W-FENCE2
  exemption) — keeps `codec::gl_offset_framed`, **bit for bit**. This is
  #2622/#2623's `−1 fnbyte-exact` and it is not being paid again.
* `codec::parse_gl` (the edit model's K2a span typing) keeps the incumbent
  framing. Its fail-closed cross-check is `values == ex_offsets_ordered`, and a
  wider framing can only turn that equality into a `BTreeSet::new()`, i.e.
  silently un-type every span. Out of scope, stated rather than discovered.
* `bind::gl_body_record_names` — the published-414 instrument — is left as it
  is so the number stays comparable, and its **precise** twin is added beside
  it as a measurement of how much of 414 is false-positive records.

**Not shipped, deliberately**: any change to `Bindings::selective`'s four
clauses. Clause 4 refuses unconditionally and this lane does not give it a
witness; the selective path still converts **0** by construction.

---

## 3. REGISTERED PREDICTIONS

Each row states the antecedent the claim actually needs — including *"and no
earlier clause fires"* where the claim is about a clause (`w-selbind`'s deciding
row split for exactly that omission) — and what would falsify it.

| id | claim | P | antecedent it needs | falsified by |
|---|---|---:|---|---|
| **C1** | `fnbyte-exact` delta is **exactly 0** (35,810 → 35,810) | 0.90 | *given* `gl_defined_names_with` and `plain_external_defined_names` keep `codec::gl_offset_framed` unchanged, **and** the relaxed framing binds the same records on every TU whose bodies the byte judge grades — which §1.2 measures as "0 of 876 TUs lose or gain a 1:1 binding" | any non-zero delta in either direction |
| **C2** *(deciding)* | **the acceptance path's bound does NOT reach 414.** `selbind-emit-subset-gate-tus`, recomputed at the SHIPPED gate framing, lands **strictly below 414** | 0.85 | *given* the gate's binding walk keeps all six stop clauses **and none of them is removed or reordered in this lane**, **and** `gl_body_record_names` — the reader behind 414 — runs no walk at all (§1.1). The claim needs both: if the walk were removed the number would be 414 by construction, and if 414 came from a walk the framing would be the only difference | the shipped-framing count reading **414** |
| **C2b** | …and it lands **at or below 60** | 0.55 | as C2, *and* `gl-stop-26-introduced` remains the first cause on ≥ 800 of the 848 refused TUs, as the base scan reads | a count of 61 or more |
| **C3** | **0 of 878 TU verdicts move** — match stays 23, mismatch 0, and no TU changes class in either direction | 0.92 | *given* the 32 TUs that are 1:1 under the gate framing are the same 32 under the relaxed one (§1.2, measured over 876 of 878), **and** the two TUs this lane could not capture are `capture-fail` at the base scan too | any class change on any TU |
| **C4** | the relaxed framing makes the gate walk stop **EARLIER** on more TUs than it makes it stop later — i.e. `gl-stop-*` cause counts move, and `gate_causes` is not invariant | 0.75 | *given* the wide record set is a strict superset (0 of 876 TUs lose a position) and each extra record is a fresh opportunity for one of the six clauses | every `gl-stop-*` count unchanged |
| **C5** | the published **414 is inflated** by not-a-split records: the same subset test under the *precise* framing reads **below 414** | 0.50 | *given* a not-a-split record still contributes its nearest-preceding run to the name set, and 406 of 876 TUs carry one | the precise count reading exactly 414 |
| **C6** | the over-emit counterexample (`inline u`, `inline v`, `int f`) still **refuses** at clause 4 under the relaxed framing, with `Port` never `Mismatch` | 0.97 | *given* clause 4 is untouched and refuses unconditionally whenever `seg_ix.len() != segs.len()`, **and** the relaxed framing does not make that TU's records 1:1 with its 3 segments. **This is the row that is nearly unlosable, so:** it is falsified by the cell grading `Port=Mismatch`, or by the records becoming 3 (which would route it through the 1:1 arm and out of clause 4 entirely) | a `Mismatch`, or a 1:1 route |
| **C7** | the full gate is green — 18/18 lanes, **0 mismatch anywhere**, including the generated corpus (`expr_sweep` + `mode_cross`), which is where `w-selbind`'s 35 wrong objs were caught and where the `/Ox` fixture scan read 0 | 0.88 | *given* C3 holds **and** the relaxed framing changes no accepted class, so every generated case either binds identically or refuses | any mismatch anywhere |

**The deciding row is C2.** If it loses, the commission's headline —
*"a one-byte frame relaxation moves the ceiling from 34 to 414"* — is correct
as stated and this lane simply ships it. If it wins, the 380-TU gap is
**#2783 plus the walk**, the successor frontier is the walk's stop clauses
rather than the framing, and `CEILING.md` §13.1 needs a third column.

---

## 4. WHAT THIS LANE DECLINES TO RE-PRICE

* `vec.cpp`'s nine mechanisms (#2827) and `decomp_pch.cpp`'s six (#2785) — read,
  not re-derived. This lane touches neither TU's codegen.
* The `.rdata$r` ladder head (#926–#933, declined twice) — untouched.
* `Bindings::selective` clause 3's open edge (#1721) — clause 4 keeps it
  unreachable and this lane does not change that.
* `codec::parse_gl`'s K2a typing — §2 states why it keeps the incumbent
  framing; the edit-model consequence of a widened framing there is unmeasured
  and is declared unmeasured.
