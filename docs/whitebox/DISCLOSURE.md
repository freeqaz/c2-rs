# DISCLOSURE — disassembly-derived findings adopted into the port

> **PROVENANCE — DISASSEMBLY-DERIVED.** This directory is the output of a static
> analysis of Microsoft's `c2.dll`. See [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0
> for the exact bytes.

## What this file is for

`README.md` currently makes a **blanket** clean-room claim: the original binary
is treated as a black box and only its observable output informs the port.
`docs/ROADMAP.md` §9.8 states the consequence precisely:

> If a disassembly-derived constant is ever adopted, that blanket claim must
> weaken to per-finding disclosure, naming the site in the relevant `docs/` file.

§9.4 previously recommended taking on **no** white-box debt. The user has now
explicitly authorized this analysis, so that recommendation is superseded for the
`w-map` lane — but the disclosure discipline is not. This file is the ledger that
makes the consequence handleable instead of quietly broken.

## Two provenance tiers — they are not the same, and the difference is cheap to keep

The lane's central artifact, the translation-unit partition, is **not uniformly
white-box**. It has two components with genuinely different provenance, and
pooling them would concede more than the work actually costs.

| tier | what | provenance | debt |
|---|---|---|---|
| **TIER 1** | **the list of 53 file names** (`coff.c`, `coffemit.c`, … — [`C2_MAP.md`](C2_MAP.md) §3A) | c2's C1001 path prints `compiler file '%s', line %d`, so these are **plain `strings` output** — an observable of the black box, recoverable without a disassembler | **none** |
| **TIER 2** | **every address**: the ICE-site xrefs, the derived per-file ranges, and all function labels | reading the disassembly | white-box |

`docs/ROADMAP.md` §9.8 already blesses tier 1's class explicitly: **the
diagnostic strings are named there as an observable output of the black box**,
alongside the obj, the `/FAsc` listing and the error text. Nothing about
extracting them requires or implies disassembly — `strings c2.dll | grep vctools`
is sufficient and is the same category of observation as reading a `C1007`
message.

**Consequence: the file-name list on its own incurs no white-box debt at all.**
A reader who only wants to know that this compiler's back end is built from
`p2\`, `p2\ppc\`, `p2\smd\` and `common\`, that EH is split across `ehexcept.c`
and `except.c`, or that `coff.c` and `coffemit.c` are separate translation
units, can have all of that from tier 1.

What tier 2 buys on top is *where* — the ranges that turn the name list into a
map. That is real white-box debt and is not minimised here. But it is worth
noting which half of the lane's headline result rests on it: the **link-order
validation** in §3.2 (7 ascending runs against 26.5 expected,
P = 1.5 × 10⁻²⁵; every run directory-pure) is a joint fact about tier 1 and
tier 2 and needs both. The **file inventory** needs only tier 1.

Keep the tiers apart in anything derived from this directory. Blurring them
costs the project more than the analysis did.

## The rule

**Navigation is free; adoption is not.**

* Using this directory to decide *where to look* in the binary, or to decide
  *which black-box experiment to run next*, costs nothing and needs no entry
  here. A map is navigation.
* **Copying a value, a table, a bit layout, or an algorithm out of the
  disassembly and into `crates/` is adoption**, and requires a row below *in the
  same commit as the code change*, naming the address it came from.
* The grey zone — a white-box finding that told you what to look for, which you
  then re-derived and confirmed purely from black-box observation — should still
  be logged, marked `route:` in the Kind column. It cost the blanket claim
  nothing, but a future reader deserves to know the search was not blind.

## Adopted findings

| # | Kind | What was adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-ALIAS-1** | **adoption** | **The `.gl` tag-0x10 ALIAS record's grammar and its discriminator bit.** The tag dispatch routes `0x04`/`0x0E`/`0x10` to one shared kind-4 handler that splits only at the end; the `0x10` arm sets `[sym+0x37] \|= 0x400000` and stores **one `varU`** into `[sym+0x4c]`, at the same anchor a tag-0x0E record puts its `.ex` body offset. So on a tag-0x10 record that word is a **symbol token**, not a flag word — which is the whole finding, and it is a *bit layout*, so it is adoption and not navigation. | `0x10b9b91f` (dispatch), `0x10b9bdcf` (shared kind-4 header), **`0x10b9c01e`** (the tag test), **`0x10b9c024`** (`\| 0x400000`), **`0x10b9c030`** (the store), `0x10b9c033` (the shared tail) | `crates/c2-il/src/func/glalias.rs` — module docs, `ALIAS_TAG`, `record_head` | `d2bdadc` | Independently confirmed against real `c2.dll` by lane `w-emitp` (15/15 interventional draws, 0/15 parity control) and reproduced by two implementations agreeing on 850 TUs. The **grey-zone alternative was tried first and is insufficient**: a black-box search for the field position binds at 0.019/0.026 one byte either side, so the position is identified by the disassembly and only *graded* by the corpus. |
| **W-ALIAS-2** | **route** | **`+0x37 & 0x400000` has exactly two readers, and the emit-relevant one resolves the token and sets `+0x20 \|= 0x2000` on the TARGET.** This is what licenses the extensional claim the port's model uses — an initializer node naming an alias contributes the alias's *target* — and it is the reason `dom(alias)` is never itself emitted. | **`0x10b99621`** (`test [esi+0x37],0x400000`), **`0x10b99635`** (`or [eax+0x20],0x2000`), `0x10b8ac60` (the second reader, `or [eax+0x32],1` — read, modelled nowhere) | `crates/c2-il/src/func/glalias.rs` — module docs only; **no value or layout is copied from these sites** | `d2bdadc` | Logged as `route:` per the grey-zone rule: the reading told this lane what the record *means*, and the meaning was then established by black-box experiment (`w-emitp` §4, real `c2.dll`) and by corpus measurement (`dom(alias) ∩ E` = 0 over 174 417 emitted names). The instruction that turns `+0x20 & 0x2000` into the COFF Mark bit is **named (`0x10b28ca3`) and NOT decoded**. |
| **W-MEMCPY-1** | **route** | **The block-move expansion decision.** `align` = the front end's alignment hint; `n = size / align` **truncating**; `inline` iff `n <= T`, `T = 5` with favor-size and `10` with favor-speed; a non-constant size is a call; a zero size and a dead non-escaping local destination emit nothing. Written into [`../IL_INTRINSIC_CALL.md`](../IL_INTRINSIC_CALL.md) §5.1.1 and pointed at from one comment in `crates/`. **No constant, address, bit position or layout is in `crates/`** — the code's behaviour is unchanged and every intrinsic is still refused. | `0x10bf65b8`, `0x10bf65d1`, **`0x10bf65e3`** (`cmp eax,5`), **`0x10bf65de`** (`cmp eax,0xa`), `0x10bf65e6`, `0x10bf657f` / `0x10bf6584`, `0x10bf658b`, `0x10bf669d`; memset's copy at `0x10bf5e30`–`0x10bf5e46`. Named here for re-checking and **not decoded into any file** | `docs/IL_INTRINSIC_CALL.md` §5.1.1; `crates/c2-il/src/func/body/expr.rs` — **one comment, which points here and states no constant** | `cc14d018` | **Logged `route:`, and `WB_MEMCPY_FINDINGS.md` §9 pre-drafted it as `adoption`. The downgrade is earned, not asserted.** `work/w-memfit/holdout.py` fits **both** constants from obj cells alone — an exhaustive search over four candidate quantities × every threshold 0..2048 — and holds them out in both directions: fitted on GRID-W's 72 `/O1` cells the rule scores **232/232 and 176/176** on `w-memcpy`'s two grids, which it was never fitted to; fitted on those 408 it scores **72/72** on GRID-W `/O1` and refuses `/O2`, `/Ox`, `/O1 /Ot` at 18/36 each. **624 of 624 across the three grids.** What the disassembly supplied is the **search space** — `size / align, truncating` is a quantity nobody enumerated before reading it, and `w-memcpy` froze six rivals over 408 cells without one of them being a quotient. That is navigation, and the grey-zone rule says log it. Reciprocally, GRID-W has **0** cells that can see the truncation (its `n` axis is exact multiples) and `w-memcpy`'s have **22**, truncating 22 / ceiling 0 — so the oracle decides a part of the reading the whitebox lane's own grid could not |

> ### **2026-08-09 — `WB_MEMCPY_FINDINGS.md` §9's other three pre-drafted rows are NOT carried, and each has a reason.**
>
> * **W-MEMCPY-2** (`0x10c2e310` is bit 23 of the option word) — **not carried.**
>   The port needs the *behaviour* (the threshold follows favor-speed, not the
>   `/O<n>` level) and that is what GRID-W measures, at 180/180 across five flag
>   sets. Nothing anywhere reads an option-word layout. A row would disclose a
>   bit position the project does not use.
> * **W-MEMCPY-3** (the callee name is minted inside c2 from a string literal) —
>   **not carried, because it was re-derived black box in this lane and needs no
>   route.** A TU whose only call is `memcpy` has `?f@@…`, `.XBLD$W`,
>   `__C1_11886` and the `/include:` directive in its `.gl` and **no `memcpy`**,
>   while its obj carries `[14] memcpy sc=EXTERNAL sec=0 type=0x0020`. That is
>   two observations of the black box's own output, and `w-memcpy` §2 had already
>   made the first of them before any disassembly existed.
> * **W-MEMCPY-4** (the removal site) — **not carried, by that document's own
>   instruction.** The rule adopted is `E-DEADDST`, obj-established at 36/36 in
>   GRID-W and 44/44 in GRID-M2, and it needs no address at all. `0x10b482ba`
>   stays `unknown`.

> ### **2026-08-09 — the `W-SELECT-*` rows are PRE-DRAFTED IN TWO PLACES WITH DIFFERENT CONTENTS, and lane `wb-selfit` reconciled them. NOTHING IS CARRIED.**
>
> `WB_SELECT_FINDINGS.md` §10 and `WB_SELECT_FINDINGS_R2.md` §9 each pre-draft
> five rows under the same five names, from two independent readings of one
> image on one day. **Ten drafts, five names, no adopted row** — no lane in that
> family has changed `crates/`, so none of them belongs in the table above yet.
> [`WB_SELECT_RECONCILED.md`](WB_SELECT_RECONCILED.md) §14.2 merges them to six;
> the operative points for whoever carries them:
>
> * **`W-SELECT-2` (the operator × type tables) — use `WB_SELECT_FINDINGS_R2.md`'s
>   version.** The other lane's enumeration is missing the thirteenth table,
>   `convert` @ `0x10b1fd08` (board **#2200**). **The black-box alternative is
>   complete and should be preferred**: the two grids plus `diag.cpp` re-derive
>   every live entry, the signedness split, `srawi`+`addze`, the `lha` fusion and
>   the absence of a magic-number multiply **with no address**.
> * **`W-SELECT-3` (the cost model and the tie rule) is the row that genuinely
>   needs an address, and the case is now STRONGER than either lane made it.**
>   Both wrote that no obj separates *"`cntlzw` was cheaper"* from *"ties go to
>   `cntlzw`"*. Board **#2204** adds that no obj in this project ever reached the
>   comparison: `FUN_10c1b517` routes an against-zero relational to
>   `FUN_10c1a908` first, and **five of the two grids' 24 cells** are exactly
>   that. Use `WB_SELECT_FINDINGS.md`'s relation-code table — the other lane's
>   has two transposed pairs (**#2207**), so the canonical form is `UGT`.
> * **A SECOND row needs an address, and it is a COUNT.** 13 tables, 41 dispatch
>   arms, 18 expansion arms. `WB_SELECT_FINDINGS_R2.md`'s `W-SELECT-4` note said
>   so first and it is upheld: **no obj yields a count of arms**, and those three
>   numbers are what both judgment rows' prices rest on. A port that only
>   *implements* the rules needs none of them.
> * **`W-SELECT-5` is downgraded and re-pointed.** Its adoption-ready clause
>   *"`&` with a contiguous mask is `rlwinm`, never `andi.`"* is over-general
>   (**#2210**); the deciding routine is **`FUN_10c0a2e2`**, not `FUN_10c1772b`
>   (**#2203**). Navigation, held.
> * **One row neither lane proposed**: `FUN_10c1a908` @ `0x10c1a908`, the
>   against-zero relational, ~20 arms, **unread by both** and the thing five
>   graded cells actually exercised. Navigation, held.

> ### **2026-08-08 — lane `w-phase7` gave W-ALIAS-1 and W-ALIAS-2 their first CONSUMER, and adopted NO new address doing it.**
>
> The `Adopted into` column of **W-ALIAS-1** should now be read as
> `crates/c2-il/src/func/glalias.rs` **plus** `IlBundle::data_tu`'s alias
> fence and `IlBundle::in_alias_report`, and **W-ALIAS-2**'s as unchanged
> (module docs only). No constant, offset, bit position or layout beyond the
> two rows above entered `crates/` in that lane:
>
> * `ObjImage::weak_externals` and `ObjImage::relocs_named` are **PE/COFF
>   format** readers — `IMAGE_SYM_CLASS_WEAK_EXTERNAL`, the weak aux record's
>   `TagIndex`/`Characteristics`, the relocation table — all published format,
>   none of it derived from `c2.dll`. **No white-box debt.**
> * The realisation rule *"c2 writes `??_E<X> → ??_G<X>` iff `??_G<X>` is a
>   `.text` COMDAT leader of the same obj"* is **extensional**, derived from
>   878 objs and graded per record (4,013/4,013, 0 miss, 0 extra). It is a
>   statement about c2's **output**, which is the black box's own observable.
>
> **And W-ALIAS-2's `route:` claim is now confirmed harder than it was.** That
> row's stated meaning — *"an initializer node naming an alias contributes the
> alias's target"* — was licensed by `w-emitp`'s 15/15 interventional draws.
> The weak-external reading is a second, independent confirmation **from the
> obj alone**, needing no mutation and no disassembly: the pairing `??_E<X> →
> ??_G<X>` is written into the symbol table where anybody can read it. A
> `route:` row whose meaning is independently visible in the output is the
> cheapest kind of white-box debt there is.
>
> **What is still NOT adopted, and what the next lane would need.** A Rust
> emit-set model needs the `.gl` **reference-list** decode
> (`work/w-refs/refs.py`), which carries `0x10b9bf99` (the list, gated on
> `flags4c & 0x1000`), `0x10b276e4` (the Mark walk) and `0x10b9be44` (the
> storage-class-`0xa` skip). **None of those three is in this ledger and none
> is in `crates/`.** `w-phase7` declined the port rather than adopt them
> silently — see `rungs/2026-08-08-w-phase7.md` §7.2, whose first named step is
> a row here.

**These are the first two rows, and `README.md` changed in the same branch** —
its clean-room claim now reads per-finding and points here, exactly as step 4 of
the checklist below requires. Everything else the `w-map` lane produced remains
navigation, not adoption.

**What is NOT adopted, stated so absence does not read as coverage.** The four
`.gl` scalar encodings the record walk needs (`0x10c1f8fc`, `0x10c1f91b`,
`0x10c1f9a6`, `0x10c1f9e9`, `0x10c1fae7`, `0x10c1f90a`, `0x10c1fcef`) are named
in comments as *navigation*: the same encodings were already re-derived from
black-box IL in `crates/c2-il/src/func/readers.rs` before any disassembly was
read, and the copies in `glalias.rs` exist only because the walk needs them at
`.gl` positions. No row is claimed for them, and if a future reader disagrees
with that call the fix is to add a row, not to remove the comment.

## If you are about to add the first row

1. Add the row *before* or *with* the code change, never after.
2. Name the address, not just the function — a future reader must be able to
   re-check your reading.
3. Say in the code comment that the value is disclosed, and point at this file.
4. Tell the coordinator: `README.md`'s wording must change from a blanket claim
   to a per-finding one at the same time. That is a one-line edit and it must not
   lag the code.
5. Prefer the alternative first: if the same fact can be established by a
   black-box experiment against the real toolchain, run it and adopt *that*
   instead. The oracle is cheap; the clean-room claim is not.
