# tools/

Shared, documented CLI tools for agents doing static analysis in this repo.
Outside the `std`-only Rust workspace (like `scripts/plot_perf.py`) — shell
and Python only, stdlib/no-external-deps, no absolute machine paths baked in.

- **`c2sym`** — query the pre-exported Ghidra flat files for `c2.dll` itself
  (decompiled C, disassembly, function/xref/call-graph rows, strings, data,
  which translation unit an address likely came from). Never opens the
  Ghidra project (concurrent access corrupts it) — reads
  `$C2RS_GHIDRA_EXPORT` (default `$HOME/ghidra-projects/export/c2`) only.
  ```
  ./tools/c2sym decomp 10b7f1ff
  ./tools/c2sym callers 10b7f1ff
  ./tools/c2sym whichmod 10b7f1ff
  ```

- **`coffdump.py`** — inspect a PPC/Xbox 360 MSVC COFF `.obj`: sections,
  symbol table with objdiff-style inferred sizes, relocations, EH funclets,
  and a hexdump/byte-diff of one symbol's bytes (`--mask-relocs` to zero the
  4-byte window at each relocation site, isolating real byte differences from
  relocation-target differences). Takes `.obj` paths as plain arguments — not
  the correctness judge (that's `c2rs diff` / `crates/c2-obj`), just a manual
  eyeballing aid for when you already know two objects differ and want to see
  why.
  ```
  python3 tools/coffdump.py sections   work/w5-ref.obj
  python3 tools/coffdump.py symbols    work/w5-ref.obj --kind F
  python3 tools/coffdump.py symbol     work/w5-ref.obj '?c4_add@@YAHHHHH@Z'
  python3 tools/coffdump.py diff       work/w5-ref.obj work/w5-port.obj '?c4_add@@YAHHHHH@Z'
  ```

## Provenance

`coffdump.py`'s COFF reader and size-inference/funclet-signature logic are
ported near-verbatim from `rb3-xenon/scripts/analysis/coffx.py`, which itself
mirrors `objdiff`'s `obj::read`. `c2sym` has no direct sibling counterpart —
`rb3-xenon`/`dc3-decomp` query `c1xx.dll`/game binaries through a live
Ghidra/pyghidra-mcp service or a SQLite DB, not a pre-exported flat-TSV
snapshot, so it was purpose-built for this repo's export layout
(`functions.tsv`, `xrefs.tsv`, `calls.tsv`, `symbols.tsv`, `decomp_all.c`,
`objdump_intel.asm`). Deliberately NOT ported: `rb3-xenon`/`dc3-decomp` also carry
hundreds of project-specific census/matching/patch scripts (decomp-percentage
tracking, vtable/RTTI recovery, symbol-map databases) that assume their own
SQLite/JSON state and a from-scratch decompilation workflow — none of that
applies here. Also not ported: any Python-side object-diff/compare tool —
the real `c2.dll` (under wibo) plus a byte-exact obj compare with
`TimeDateStamp` zeroed is the sole judge of the port (see repo `CLAUDE.md`),
and that lives in `crates/c2-obj` / `c2rs diff`, not here.
