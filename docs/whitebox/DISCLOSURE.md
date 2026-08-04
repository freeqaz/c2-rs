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
| — | — | *(none)* | — | — | — | — |

**Empty is the expected state for a map, and it is a good state.** The `w-map`
lane produced navigation, not adoption: no constant, table, or algorithm from the
disassembly has been copied into `crates/`. As long as this table has no rows,
`README.md`'s clean-room claim is intact in substance and needs only the
scope note proposed in [`README_DELTA.md`](README_DELTA.md).

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
