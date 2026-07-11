# c2-rs docs

Format and codegen knowledge backing the port, recovered by **differential
observation** of the real toolchain (`cl.exe`/`c2.dll` 16.00.11886.00 under
wibo): compile controlled fixtures, diff the objs, classify every byte as
constant vs derived. Cross-referenced against the static-RE docs in
`../dc3-decomp/msvc-src/docs/` (`IL_FORMAT.md`, `COLOR_RE.md`) where noted.

Scope of each doc is the **MVP function class** — a single straight-line
integer-arithmetic leaf function, no calls/branches/relocations — unless a
section says otherwise. Facts marked CONST/DERIVED were verified across at
least two fixtures; the standing confirmation of the whole set is the
differential harness itself (`port(IL) == c2(IL)` byte-exact, timestamp
normalized).

- `OBJ_FORMAT_MVP.md` — COFF file layout: header, section headers, raw-data
  contents (`.drectve`, `.debug$S`, `.XBLD$W`), symbol table, string table,
  the COMDAT checksum algorithm, and the constant-vs-derived classification
  of every field.
- `CODEGEN_PPC_MVP.md` — PPC big-endian instruction encoding (`add`, `blr`),
  the X360 integer ABI, the COLOR allocator's observed scratch-register
  order, and the non-commutative hazard list (what NOT to generalize).
- `IL_BUNDLE_MVP.md` — the c1xx→c2 IL bundle (`_CL_*` ×5): capture recipe,
  token-width detection, `.gl`/`.sy`/`.ex` parse for the MVP class, and
  which bundle fields the emitter actually consumes.
- `EDIT_MODEL_MVP.md` — the K3a length-consistent `.ex` edit primitive: the
  `IlModel` mutation API (literal widen/narrow, term insert/delete), the `.gl`
  body-start offset re-emit it discharges, the fail-closed boundary
  (whole-function add/remove is K3b), and the differential edit gate that
  verifies each P0.6a experiment byte-exact as an *edit*.
- `CORPUS_MVP.md` — the P1.2 `(source, IL, obj)` triple corpus: the generator
  (`c2rs corpus`), the manifest schema, deterministic source generation, and
  the committability handling (generated corpus gitignored; synthetic sample
  committed).

Scratch artifacts referenced in these docs (`/tmp/...` fixture objs, captured
bundles) are session-local and **regenerable**: `c2rs capture <fixture.cpp>`
re-captures a bundle; compiling any fixture with `/Ox /GS- /c` under the
reference toolchain regenerates the objs. Nothing binary is committed.
