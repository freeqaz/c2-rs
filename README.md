# c2-rs

**Clean-room, I/O-behavioral native port of the MSVC Xbox 360 PPC compiler
backend `c2.dll`, plus the differential test harness that grades it.**

> **STATUS: P0.1 GREEN; native port byte-exact on the MVP function.**
> Standalone-c2 IL **replay (P0.1)** is **proven** — feeding a captured IL
> bundle back through `c2.dll` alone (via the `c2host` stub under wibo)
> reproduces the pipeline `.obj` **byte-for-byte** on all fixtures, so the
> differential's **reference side is real**. The native **port** (`PortC2`)
> now emits a **byte-exact** `.obj` for its first function class — a
> straight-line integer add-chain leaf, `int add3(int,int,int){return a+b+c;}`
> — matching real `c2` on timestamp-normalized bytes through the full harness
> (IL capture → `mvp_function` parse → PPC codegen → 5-section COFF; the
> reference's `-Fo` path is threaded in for the `.debug$S` S_OBJNAME). Anything
> **outside** that class (multi-function TUs, branches, calls, relocs) still
> returns `NotImplemented` — that is the open gate. The oracle self-test
> (determinism + IL-capture stability) also runs green.

## Why

This is [il-witness **angle H**](../decomp-synth/docs/plans/il-witness/02_ANGLES.md):
compile latency prices every loop in the decomp-synth project at once (search
moves, preimage checks, LoRA pass@k, corpus generation, candidate scoring). A
10–1000× faster `c2` multiplies every angle simultaneously. The port does **not**
need to reproduce c2.dll's own bytes — only its *behavior*:

> for every IL bundle, `port(IL) == c2(IL)` byte-exact (COFF timestamp zeroed).

The real `c2` stays resident under [wibo] as the differential judge; the corpus
generator supplies unlimited free test vectors. The criterion is **I/O
equivalence, not source fidelity** — this is what makes it tractable (there is
no objdiff target for c2.dll itself). Verification is coverage-bounded
differential testing, so corpus breadth is load-bearing. See
[`03_ROADMAP.md`](../decomp-synth/docs/plans/il-witness/03_ROADMAP.md), track
**T-E** and gate **P0.1**.

## Workspace map

| Crate | Role |
|-------|------|
| `crates/c2-il` | IL bundle **container** model — the 5 files (`ex/gl/sy/in/db`) as raw bytes keyed by suffix; load/write/round-trip; `.ex` magic + token-width heuristic. NOT a disassembler. |
| `crates/c2-obj` | COFF `.obj` handling for the differential compare — `normalized()` (zero the 4-byte TimeDateStamp at offset 4), `diff()`, `timestamp()`. |
| `crates/c2-core` | The port itself — the `Backend` trait and `PortC2`, which is **byte-exact on the MVP add-chain class** (`codegen` PPC selector + `coff::emit_mvp_obj` 5-section builder) and `NotImplemented` outside it, plus a `passes/` tree documenting the pass order and first-port targets. |
| `crates/c2-reference` | Drives the **real** `cl.exe`/`c2.dll` under wibo — the oracle. `compile_obj` (normal `/Ox /GS- /c`), `capture_il` (`/Bd /d2nop` early-abort trick), `capture_reference` + `replay` (**P0.1**, byte-exact standalone-c2 replay via the `c2host` stub), and `ReferenceC2` (bundle→obj Backend). |
| `crates/c2-harness` | The benchmark + `c2rs` CLI. `differential()` proves the reference replay is byte-exact then reports the port status (`Match` on `mvp_add3.cpp`, `NotImplemented` on the multi-function `add3.cpp`), and the live-today `oracle_selftest()`. |
| `c2host/` | Tiny x86 Windows stub (`c2host.c`) that `LoadLibrary`s `c2.dll` and calls its `_InvokeCompilerPass@12` export — the mechanism behind standalone-c2 replay. Built on demand into a gitignored cache; the `.c` is tracked, the `.exe` never is. |
| `fixtures/cpp/` | Include-free C++ TUs. **Only the `.cpp` is tracked** — IL and obj are regenerated at test time, never committed. |

Dependency edges: `c2-core → {c2-il, c2-obj}`; `c2-reference → {c2-il, c2-obj,
c2-core}`; `c2-harness → all four`. **std only, zero external crates.**

## Toolchain location (no absolute paths in source)

The reference toolchain is found via env overrides, each with a
relative-to-repo-root default (repo root = the `c2-rs` checkout; the defaults
assume the sibling-repo layout `milohax/{c2-rs, wibo, dc3-decomp}`):

| Env var | Default (relative to repo root) | Required? |
|---------|-------------------------------|-----------|
| `C2RS_WIBO` | `../wibo/build/release/wibo` | yes |
| `C2RS_WIBO_DEBUG` | `../wibo/build/debug/wibo` | no |
| `C2RS_CL_EXE` | `../dc3-decomp/build/compilers/X360/16.00.11886.00/cl.exe` | yes |
| `C2RS_C2_DLL` | `../dc3-decomp/build/compilers/X360/16.00.11886.00/c2.dll` | yes |
| `C2RS_DC3_ROOT` | `../dc3-decomp` | no |

`Toolchain::locate()` returns `None` if any required path is missing, and every
test and CLI subcommand then degrades to a clean `SKIP: toolchain absent`.

## How IL capture works

wibo maps `Z:\` → host `/` and runs `cl.exe` natively. A normal compile is
`wibo cl.exe /Ox /GS- /c /Fo<obj> <src>`. To capture IL, add `/Bd /d2nop`: c2
aborts *before* deleting the temp `_CL_*` bundle, so the 5 files
(`ex gl sy in db`, no dot) survive. The bundle base is scraped from the
`-il <...>_CL_<hash>` token in the compiler output. The `/Bd /d2nop` compile
exits **non-zero** — that is success; the `_CL_*ex` file's presence is the real
signal. (`TMP`/`TEMP` are pointed at a private work dir so the bundle lands
deterministically.)

## Running

```sh
# Build + test the whole workspace (integration tests skip if toolchain absent):
cargo build --workspace
cargo test  --workspace

# The live-today benchmark — determinism + capture stability over all fixtures:
cargo run -p c2-harness --bin c2rs -- selftest

# P0.1 replay — capture + standalone-c2 replay, byte-match verdict:
cargo run -p c2-harness --bin c2rs -- replay fixtures/cpp/add3.cpp

# Other subcommands:
cargo run -p c2-harness --bin c2rs -- capture fixtures/cpp/add3.cpp
cargo run -p c2-harness --bin c2rs -- compile fixtures/cpp/add3.cpp
cargo run -p c2-harness --bin c2rs -- diff    fixtures/cpp/mvp_add3.cpp # ReferenceReplay=ByteExact, Port=Match
cargo run -p c2-harness --bin c2rs -- diff    fixtures/cpp/add3.cpp   # ReferenceReplay=ByteExact, Port=NotImplemented (multi-function)
cargo run -p c2-harness --bin c2rs -- bench

# Performance benchmark (angle H) — IL-bundle -> obj latency, port vs real c2:
cargo run -p c2-harness --bin c2rs -- perf
cargo run -p c2-harness --bin c2rs -- perf --port-iters 5000 --ref-iters 10 --fixtures mvp_add3.cpp,mvp_sub.cpp
```

The `replay`/`diff`/`perf` paths additionally need `strace` (keeps the IL
bundle) and `i686-w64-mingw32-gcc` (builds the `c2host` stub); both degrade to a
clean `SKIP` when absent.

## Performance — the angle-H payoff

`c2rs perf` measures the whole thesis directly: the per-obj latency of turning a
captured IL bundle into a `.obj`, the port vs the real backend. Both sides
produce the **same** obj from the **same** bundle —

* **port** — `PortC2::compile_to`, pure in-process Rust (parse IL → PPC select →
  emit COFF), and
* **reference** — `Toolchain::replay`, standalone `c2.dll` under wibo (spawn
  `wibo c2host c2.dll …`).

Each fixture is captured once; the port's obj is confirmed **byte-exact** to the
reference before timing (so equal output is being compared, not a shortcut), and
each side is timed for `N` iterations (median + mean). Fixtures outside the
ported class time only the reference and report `NotImplemented`.

On the bundled fixtures the in-process port is **~200–270× faster** per obj than
standalone c2 (single-digit-µs vs ~4–6 ms) — the throughput multiplier that
prices every decomp-synth loop. Sample row:

```
  fixture                          obj     ref median    port median      speedup  port
  mvp_add3.cpp                    842B       4.522 ms       17.81 µs         254x  Match
  ...
  geomean speedup over the 12 matched fixture(s): ~235x faster than standalone c2
```

Numbers are machine-dependent; run it on your box for the real figure. `perf`
reports a port `Mismatch`/`NotImplemented` per fixture but only *fails* (non-zero
exit) on a capture/replay error or a broken P0.1 replay — the reference stays
the sole judge.

[wibo]: https://github.com/decompals/wibo
