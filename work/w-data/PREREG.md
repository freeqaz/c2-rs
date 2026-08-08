# w-data — pre-registration

Lane `w-data`, worktree branch `wt-w-data` off master **`536112b8`**.
Baseline: match 12 · mismatch 0 · codegen-gap 0 · vocab-gap 859 · capture-fail 7
· FRONTIER 15 · tests 1,252 / 36 targets · gate 5,238 fixture-verdicts.

**Committed before the first `cl.exe`, the first probe capture and the first
line under `crates/`.** Everything below was written from a code read of the
base tree and from `docs/rungs/2026-08-08-w-cfg2.md` / `w-cfgclass.md` /
`work/w-cfg2/PRIMES_BODY.md`. No obj has been produced by this lane yet.

---

## §0 The target and the ladder

`src/system/math/Primes.cpp` — one blocked function `?NextHashPrime@@YAHH@Z`,
64 B `.text`, 6 relocations, one function-local `static int primes[62]`
(248 B, `.data` COMDAT, STATIC symbol `?primes@?1??NextHashPrime@@YAHH@Z@4PAHA`).

The brief's ladder, fenced at every rung:

* **(a)** the READER channel — a data object as a first-class thing the IL
  parser produces, accepted in the parser (#139) not the emitter.
* **(b)** `.data` COMDAT emission + a DEFINED STATIC symbol + the REFHI/REFLO
  pair against it, byte-exact on Primes.
* **(c)** the sixteen `.text` words (`codegen::frontier_bytes` already builds
  them under `cfg(test)`; a production carrier is what is missing).
* **(d)** `undname` / `osfinfo` / `vswprnc` — ONLY if a re-survey says their
  remainder is this seam.

---

## §1 The seven refusals — my reading at THIS base, before measuring

Read from the code at `536112b8`. Each is scored in the rung.

| # | w-cfg2's claim | my reading of the base tree | expected outcome |
|---|---|---|---|
| 1 | `gl_data_objects_ordered` reads the COMDAT attribute; `data_tu` refuses COMDAT | **CONFIRMED in code.** `gl.rs:1141 DATA_ATTR_COMDAT = 0x20`, `data_object_at` sets `comdat`, `GlDataObject::comdat` exists. The refusal is in `IlBundle::data_tu` | the record parses; `data_tu` is the WRONG consumer for Primes anyway (its whole class is *functionless* TUs) — I expect to leave `data_tu`'s refusal **untouched** |
| 2 | `gl_extern_data_names` refuses linkage `04` | **CONFIRMED.** `gl.rs:1512`, `LINKAGE_UNDEF_EXTERN = 0x02` required; `01` and `04` explicitly refused | OPEN; needs a **second** resolver, not a widening of this one |
| 3 | the unclaimed-`.gl`-name gate refuses the TU | **CONFIRMED.** `bundle.rs:1639`, `bind.unclaimed` vs `accounted`; the file-offset-2 comment is at `bundle.rs:1602` | OPEN; closed by accounting the *defined* object |
| 4 | `IlFunction` has no channel for a data OBJECT | **CONFIRMED.** `mod.rs:2006 data_sym: Option<String>` is documented as an **undefined external**; nothing carries size/align/bytes/COMDAT | OPEN — the structural one |
| 5 | R3: one REFHI with TWO REFLOs | **CONFIRMED in the writer.** `coff::DataRef { hi_off, lo_off, name }` is 1:1 and both writers hard-code `4 * data_refs.len()` records | OPEN |
| 6 | relocation against a STATIC symbol | **CONFIRMED.** both writers emit `emit_external_symbol(name, 0, 0x0000)` — section 0, UNDEF | OPEN |
| 7 | the slot is the COMDAT bit, not #1179's first-referrer test | **the writer has no assumption to check** — `emit_comdat_obj` has no `.data` path at all. `emit_data_obj` (the functionless writer) has the S1′ slot logic and is unreachable from a TU with functions | OPEN as *absent*, not as *wrong* |

### §1.1 An EIGHTH item the re-price does not name, found in the code read

**`codegen::labels::LabelMap` invariant 4 refuses a backward reference**, and
Primes' back edge at `0x2c → 0x14` is one. I register now, before measuring,
that I expect this to be **NOT a blocker**, for two reasons already in the tree:

* `ptr_walk_loop` and `ptr_walk_chain_loop` both emit a backward `bc` and reach
  `Selected::Plain` — neither routes through the map (labels.rs header,
  correction 2);
* `Primes.cpp` is **label-free**: its one function is a leaf with no frame, so
  `plan_labels` returns `None` and the counter's value never reaches the obj
  (w-loop's Q2, 34 of 34).

**P0.** Invariant 4 is not touched and not relaxed by this lane, and the
back edge is emitted by computing the displacement directly.

---

## §2 Predictions

Each is scored RIGHT / WRONG / HALF in the rung.

* **P1 — the seven reproduce.** All seven of §1 reproduce at this base with
  the outcomes in the right-hand column; **at most one** turns out already paid.
  (Registered against the possibility that the 3-hour-old re-price is stale.)

* **P2 — the eighth is not a blocker.** §1.1's `P0` holds; the lane relaxes no
  `labels.rs` invariant.

* **P3 — the reader is where the lane's line count goes, and it is ONE linear
  pattern.** `PRIMES_BODY.md` decoded the body as one linear sequence with no
  value merge at a join; I predict the recognizer is a token-pattern production
  in `ptr_walk_loop`'s tradition, **≤ 700 lines including tests**, and that no
  general basic-block IR is built. (This is w-cfgclass's D2, restated as a
  positive prediction because that lane's D2 correctly did not fire.)

* **P4 — the `.data` bytes need NO new reader.** w-cfg2 measured `.in` token
  `0xea09` returning 248 bytes byte-identical to the reference `.data`, records
  38/38, residue 0. I predict this reproduces at my base **unchanged** and that
  I add **zero** lines to `ininit.rs`'s decode.

* **P5 — the section slot is APPEND.** In `emit_comdat_obj` both `.XBLD$W`
  watermarks are in the shell prefix and the code groups follow, so "after the
  code groups" is a plain push at the end of the section vector. I predict the
  reference obj's section order is
  `.drectve .debug$S .XBLD$W .XBLD$W .text .data` — **6 sections** — and that
  no shell section moves.

* **P6 — the relocation record count is 6 and the shape is
  REFHI/PAIR/REFLO/PAIR/REFLO/PAIR.** One REFHI at `0x00`, REFLOs at `0x08` and
  `0x0c`, each followed by a PAIR whose symbol field is 0, sorted ascending by
  VirtualAddress. The `.data` section itself carries **0** relocations.

* **P7 — the data symbol is a DEFINED STATIC**: StorageClass 3, section = the
  `.data` COMDAT's number, `Value` 0, `Type` 0x0000; and the `.data` COMDAT
  carries `Characteristics 0xC0301040` with aux `Selection = 2` (ANY) — the
  values GRID A cell a1 read off c2's own obj.

* **P8 — the `.data` COMDAT's aux CheckSum is a real CRC-32.** Rule D1
  (`data.rs:429`) is stated for a *non-COMDAT* `.data`; every other COMDAT this
  port emits carries 0 except `.pdata`. I register the **non-obvious** call:
  the COMDAT `.data`'s aux `CheckSum` is the real `coff_checksum` of its 248
  bytes, not 0. Confidence 0.6 — this is my least-confident prediction and it
  is exactly the byte a wrong guess costs an obj on.

* **P9 — THE CONVERSION CALL: match 12 → 13.** `Primes.cpp` converts.

  **The calibration, cited as the brief requires.** Board #770 is ~12-to-1
  optimistic on forward estimates — which argues *against* this call. Against
  that: the last two lanes' conversion calls were **both wrong**, in opposite
  directions (w-cfgclass registered no-conversion and converted; w-cfg2
  registered conversion and declined), so the prior on the *call* is worse than
  the prior on the *estimate*. What tips me is that three of the four ladder
  rungs are already built and **graded by the oracle**: the 16 words are
  oracle-asserted in `frontier_bytes`, the 248 `.data` bytes are oracle-verified
  byte-identical out of `.in`, and the `.gl` record parses. The unbuilt work is
  plumbing plus one recognizer, and w-cfgclass established that a transcription
  of one block plan is one production.

  Registered at confidence **0.6**, not higher, because P8 and the symbol-table
  ORDER of a `.data` COMDAT group in a `/Gy` obj are both single-cell facts I
  will read off exactly one obj.

* **P10 — REGISTERED BIAS.** If P9 is wrong, it will be wrong on the **WRITER**
  half — specifically the symbol-table order/indices of the `.data` COMDAT
  group inside `emit_comdat_obj` — and **not** on the reader. w-cfg2's P8
  registered the mirror of this and was right; I am registering that the halves
  have swapped, because the reader's three refusals are now all *named* and the
  writer's group order is *unmeasured*.

* **P11 — the stretch does not land.** `undname` / `osfinfo` / `vswprnc` do
  **not** convert in this lane. Registered explicitly so a clean stop after
  Primes is scored as the predicted outcome rather than as a shortfall.
  (`undname` and `osfinfo` each need two data symbols per body AND the `lis` not
  being the body's first word, which `data_refs_of` requires; `vswprnc` needs a
  REFHI/REFLO against a **code** symbol.)

* **P12 — the gate.** mismatch 0 everywhere, codegen-gap 0, gate 18/18 with 0
  NO-RESULT, `cargo test --workspace --release` 0 failed. No previously-emitted
  obj changes: every one of the 12 matching TUs is still a match, by name.

* **P13 — zero `DISCLOSURE.md` rows.** The listing seam (`/FAsc`), the
  reference obj and `.gl`/`.in` are sufficient. The last three
  conversion-adjacent lanes needed zero.

---

## §3 Decline clauses — frozen thresholds AND SIZES

w-cfg2's D1 lesson, applied: **each clause names the SIZE of the thing it
declines, not merely its presence.** A clause that fires must say how big the
declined thing measured, so the next lane can price it.

* **D1 — the reader channel.** Decline if carrying a defined data object from
  `.gl`/`.in` to the writer needs a **new IL container file** to be read (one
  `IlBundle::get` key this tree does not already consume), or if it needs more
  than **3** new public fields across `IlFunction` + one new struct. Does NOT
  fire for: a new field, a new struct, a new resolver function, or a new
  accounting line in the unclaimed gate. *Size to report if it fires: the
  number of new container keys and the number of new fields.*

* **D2 — the recognizer.** Decline if the body production needs a general
  basic-block IR — i.e. if the token pattern cannot be matched by a linear walk
  and needs a worklist over a successor graph. Threshold: **> 900 lines** of
  new recognizer code excluding tests, or any `struct BasicBlock`. *Size to
  report: the line count reached before stopping, and the specific token
  position the linear walk could not handle.*

* **D3 — the writer's symbol-table order.** Decline if the reference obj's
  `.data` COMDAT symbol group cannot be placed by a rule that **≥ 3 graded
  cells** agree on. One obj is one cell; GRID A's a1/a2/a3 are three more that
  already exist as sources and can be re-cut. *Size to report: how many cells
  were cut, and which orderings they failed to separate.*

* **D4 — the relocation fan-out.** Decline if the 1:2 REFHI/REFLO fan-out
  cannot be derived from the class (i.e. if which REFLO goes where depends on
  something the recognizer cannot see). Threshold: **≥ 2** of GRID A's a1/a4/a5
  cells must reproduce the 1:2 shape and agree on the record order. *Size to
  report: how many of the three reproduced and where they disagreed.*

* **D5 — THE FENCE.** If widening anything makes a previously-emitted obj
  change, STOP and restore. `mismatch` going 0 → nonzero anywhere is an alarm,
  not a gap (#232), and a refusal becoming a wrong emit is strictly worse than
  a gap. Threshold: **any** of the 12 matching TUs losing its match, or any
  gate lane reporting a mismatch. *Size to report: the TU name and the file
  offset of the first differing byte.*

* **D6 — the census/gate symmetry.** If the new class's accept gate lands in
  the emitter instead of the parser, `census_gate.rs` goes red (w-cfgclass
  §5.3). That is a **fix-in-lane**, not a decline — registered here so it is
  not mistaken for one.

---

## §4 What this lane will NOT do, registered up front

* It will **not** widen `data_tu`'s COMDAT refusal. That fence is load-bearing
  (w-cfg2 §3.3) and `data_tu`'s class is functionless TUs, which Primes is not.
  If I touch it, every previously-emitted obj must be proven unchanged.
* It will **not** relax `labels.rs` invariant 4 (§1.1).
* It will **not** admit the `0x40` `selectany` attribute bit.
* It will **not** reopen `expr-op-0x27` or `assign-store-type-8643` as families
  (BOARD.md excludes both).
* It will **not** carry w-cfgclass §5.3 / board #1638's `ptr_walk_loop`
  parser-half item unless this lane's own path touches that class's accept
  boundary. w-cfg2 declined it for the same reason and the reason is unchanged:
  `ptr_walk_loop` is the class behind a **matched** TU.

---

## §5 Board rows

Range **#1700–#1719**. Unused numbers left explicitly unminted in the rung.
