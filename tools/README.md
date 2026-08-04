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

- **`probe.py`** — the whole measurement loop in one command: compile a C++
  probe with the real toolchain, capture its IL, and print **one** consolidated
  COFF report. The section table, then — grouped under each section, in section
  order — the symbols that section defines (`Value`, objdiff-inferred size,
  storage class, optional COMDAT aux) and its relocations rendered **by target
  symbol name**, then the undefined externals and absolutes. Replaces three
  `coffdump.py` invocations plus the manual cross-referencing between them;
  imports `coffdump.py` rather than duplicating its reader. Takes a path, `-`
  for stdin, or `--source 'text'`. Absent toolchain → `SKIP: toolchain absent`,
  exit 0.
  ```
  python3 tools/probe.py fixtures/cpp/il_dyninit_static.cpp
  python3 tools/probe.py --source 'static int g; int f(){return g;}' --aux
  python3 tools/probe.py src/system/obj/Task.cpp --flags-file work/flags.txt --cwd ../dc3
  python3 tools/probe.py --selftest
  ```
  Two things it states that the hand-rolled loop hid. **`c2rs compile` and
  `c2rs capture` do not use the same flags** — `compile` defaults to
  `/O1 /Oi /EHsc /GS- /c` and honours `--flags-file`, `capture` is pinned to
  `/Ox /GS- /c` and takes none — and `/Ox` does not imply `/GF`, so a TU with a
  string literal captures `.gl` with no `??_C@` record while its obj carries a
  `.rdata` COMDAT for one. `probe.py` banners the skew and confirms that
  specific consequence from the two files. And `IMAGE_REL_PPC_PAIR`'s
  `SymbolTableIndex` is **not** a symbol index; rendered by name it reads as
  `@comp.id` on every c2 obj, so it is printed as a displacement.

- **`glorder.py`** — the `.gl` record-order reader, and the `.bss` relation.
  `docs/OBJ_DATA_BSS_SHAPE.md` §5.2 (Rule A1) is the load-bearing correlation in
  the `.bss` work: *eager* objects (no dynamic initializer) lay out in `.gl`
  symbol-record order, *deferred* ones in the exact reverse, and no eager object
  sits above a deferred one. Takes a `.gl`, or a `.cpp` it captures itself; with
  `--obj` it prints the three orders side by side (`.gl`, section
  ascending-address, section symbol-table) and states which relation holds.
  The record scan is a Python port of
  `crates/c2-il/src/func/gl.rs::gl_data_objects` and its `data_object_at` frame
  check, so a function or type-table record is excluded structurally rather than
  by guessing from the name — keep the two in step, `gl.rs` is the reference.
  ```
  python3 tools/glorder.py work/probe/il/_CL_*.gl
  python3 tools/glorder.py work/probe/il/_CL_*.gl --obj work/probe/probe.obj
  python3 tools/glorder.py fixtures/cpp/il_dyninit_static.cpp --obj work/x.obj --raw
  python3 tools/glorder.py ... --obj ... --section '.CRT$XCU'   # A1 must NOT hold
  python3 tools/glorder.py --selftest
  ```
  An `--obj` whose section names do not intersect the `.gl` names **at all** is
  an error, not a rule that vacuously holds.

- **`census.py`** — query the committed 871-object section census
  (`work/w-bss/census/sections.jsonl`; tracked despite `/work` being ignored, so
  this answers offline with no toolchain). One JSON record per workload object:
  source path, section count, the full **ordered** section-name list with the
  two `.XBLD$W` watermarks distinguished as `:C2`/`:C1`, and header/symbol
  detail for every `.data` and `.bss`. Filters (`--has`, `--not-has`,
  `--count NAME=LO..HI`, `--nsec LO..HI`, `--src-re`, `--before A=B`,
  `--after A=B`, `--straddles A=B`) combine with AND across every subcommand.
  ```
  python3 tools/census.py names
  python3 tools/census.py count --has .bss --after '.bss=.XBLD$W:C1'
  python3 tools/census.py count --straddles '.bss=.XBLD$W:C1'
  python3 tools/census.py list  --count '.data=2..' --sort -nsec --limit 20
  python3 tools/census.py order src/system/meta/Sorting.cpp
  python3 tools/census.py sections src/system/utl/DeJitter.cpp
  python3 tools/census.py --selftest
  ```
  A section name that never appears anywhere in the census is treated as a
  **typo and refused**, because `--has .txet` would otherwise return 0 objects
  and read as a finding. A filter that legitimately matches nothing says so on
  stderr and exits 1, so it cannot be confused with a census that failed to
  load.

## Self-tests

Every tool here has `--selftest`, it runs with **no toolchain**, and it exists
for one specific failure: this project has been bitten by instruments that
report green from an *absence* (a `sed` that read a missing number as `0` and
passed a run that graded nothing). So the checks are mostly **refusals** —
missing/empty/truncated input, a `.gl` without its header prefix, a census
record whose `nsec` disagrees with `len(order)`, a `c2rs` that exits 0 and
writes no obj, a `.gl`/`.bss` name intersection of size zero — each paired with
true positives so that a refusal is not green from the tool never working at
all. Run all three before trusting a number one of them printed:

```
for t in probe glorder census; do python3 tools/$t.py --selftest || echo "$t FAILED"; done
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

`probe.py`, `glorder.py` and `census.py` are new here — the loops they automate
existed only as `python3 - <<EOF` heredocs retyped each session, and as scratch
readers that were deleted with the worktree that wrote them. `probe.py` imports
`coffdump.py`'s reader; `glorder.py` and `census.py` import `probe.py` for the
repo-root/`c2rs`-locating/`Fail` plumbing, so there is one copy of each.

**`glorder.py` duplicates logic that is authoritative in Rust.** Its record
scan, `data_object_at` frame check, `read_token_var` and `read_varint` mirror
`crates/c2-il/src/func/gl.rs`. That is deliberate — a Python instrument must not
require building the workspace, and it is read-only — but it means the two can
drift. `gl.rs` is the reference; if they disagree, `gl.rs` is right and this is
the bug. Nothing here is a gate, and none of it may become one: the real `c2`
plus a byte-exact obj compare remains the sole judge.
