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
