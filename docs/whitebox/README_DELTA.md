# README_DELTA — proposed wording change for `README.md`

> **PROVENANCE — DISASSEMBLY-DERIVED context.** This file belongs to the
> `docs/whitebox/` lane. It proposes wording only; **`README.md` is not edited by
> this lane** — the coordinator owns it.

## Why a change is needed

`README.md` lines 31–34 currently read:

> There is no attempt to reproduce c2.dll's own code, and no decompiled source
> anywhere in the port — the original binary is treated as a black box and its
> observable output as the spec. The real `c2.dll` stays resident under wibo as
> the judge, and the port never grades itself.

Two of those clauses are still exactly true and one is now too broad:

* **"no attempt to reproduce c2.dll's own code"** — still true, and it is the
  clause that actually matters. The criterion remains I/O-behavioral.
* **"no decompiled source anywhere in the port"** — still true. Nothing under
  `crates/` came from disassembly; `docs/whitebox/DISCLOSURE.md` is empty.
* **"the original binary is treated as a black box"** — this is now too broad as
  a statement about the *repository*. A static analysis of `c2.dll` has been run
  under explicit authorization and its navigational output is checked in under
  `docs/whitebox/`. The **port** is still black-box-derived; the **repo** is no
  longer exclusively so.

Leaving the blanket sentence in place would be the bad outcome: a reader would
find `docs/whitebox/` and conclude the README is stale or untrustworthy, when in
fact the substantive claim is intact.

## The concession is smaller than it first looked — two tiers, not one

The lane's central artifact is **not uniformly white-box**, and it would be a
mistake to concede as though it were. [`DISCLOSURE.md`](DISCLOSURE.md) now
separates:

* **Tier 1 — the list of 53 original source file names.** c2's C1001 path prints
  `compiler file '%s', line %d`, so these are **plain `strings` output**:
  an observable of the black box, in the same category as a `C1007` message.
  `docs/ROADMAP.md` §9.8 **already blesses that class explicitly**, naming the
  diagnostic strings alongside the obj and the `/FAsc` listing. Extracting them
  needs no disassembler. **This tier costs the clean-room claim nothing.**
* **Tier 2 — every address**: the ICE-site cross-references, the derived
  per-file ranges, and all function labels. This is genuine white-box work and
  is not minimised.

So the README's concession need only cover *tier 2*. A reader can be told
truthfully that the back end's own diagnostic strings name its 52 source files —
`p2\`, `p2\ppc\`, `p2\smd\`, `common\` — **without that being a white-box claim
at all**. What the disassembly bought on top is *where each file lives*.

If the coordinator wants the narrowest honest wording, that is the seam to cut
along.

## Proposed replacement

> There is no attempt to reproduce c2.dll's own code, and **no decompiled source
> anywhere in the port** — the original binary's observable output is the spec,
> the real `c2.dll` stays resident under wibo as the judge, and the port never
> grades itself.
>
> One scope note. A static-analysis map of `c2.dll` lives under
> `docs/whitebox/` — addresses, string cross-references, and cluster labels, kept
> as a **navigation aid** for deciding which black-box experiment to run next.
> Nothing from it has been adopted into `crates/`:
> `docs/whitebox/DISCLOSURE.md` is the ledger of any disassembly-derived value
> that ever is, and it is **empty**. The port's correctness claim rests on
> `port(IL) == c2(IL)` and on nothing else.

## Minimal alternative

If the coordinator prefers not to grow the intro, the smallest correct edit is to
delete the five words *"the original binary is treated as a black box and"* and
add a one-line pointer at the end of the paragraph:

> A disassembly-derived navigation map of `c2.dll` lives in `docs/whitebox/`;
> nothing from it has been adopted into the port (see
> `docs/whitebox/DISCLOSURE.md`).

## Knock-on edits the coordinator should make at the same time

* `docs/ROADMAP.md` §9.4 — "take on no white-box debt" is superseded for this
  lane by explicit user authorization. It should say so *without* rewriting the
  dated record: append a note, do not edit the original sentence.
* `docs/ROADMAP.md` §9.8 — its conditional ("*if* a disassembly-derived constant
  is ever adopted…") is still the governing rule and needs no change. Point it at
  `docs/whitebox/DISCLOSURE.md` as the place the disclosure goes.
* `docs/ROADMAP.md` §9 line 4074 — the assertion that `c2.dll` "**is not a
  stripped build**" is **wrong** and should be corrected. See `C2_MAP.md`
  §"Is it stripped?". The evidence originally offered for it (the `/FAsc`
  listing) is real but supports a different claim: c2 is unusually *talkative*,
  not unusually *symbol-rich*.
