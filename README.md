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

This is the whole thesis in one picture: how fast can each backend turn a
captured IL bundle into a `.obj`? Both sides produce the **same** obj from the
**same** bundle, byte-for-byte —

* **port** — `PortC2::compile_to`, pure in-process Rust (parse IL → PPC select →
  emit COFF), no process, no shared state, and
* **reference** — `Toolchain::replay`, standalone `c2.dll` under wibo (a
  `wibo c2host c2.dll …` process spawned per obj).

![port vs c2 throughput and speedup](docs/perf/perf_scale.png)

Two consequences fall out, and both matter for [il-witness angle
H](../decomp-synth/docs/plans/il-witness/02_ANGLES.md) (verifier throughput):

* **Latency.** Per obj the in-process port is **~200–290× faster** — single-digit
  µs vs ~4–6 ms. `c2rs perf` confirms it *and* checks the port's obj is
  byte-exact to real c2 on every fixture before timing (equal output, not a
  shortcut). On the bundled fixtures: geomean **~235×**.
* **Concurrency.** The port has no per-obj process and no shared state, so it
  scales across cores nearly linearly — **~897k objs/sec** at 32 threads on this
  host — while standalone c2 pays a `wibo` spawn per obj and saturates around
  **~3.1k objs/sec**. The gap *widens* with parallelism (229× → 289×): every
  decomp-synth search that fans out preimage checks is priced against the blue
  line, not the orange one.

Reproduce on your box (numbers are machine-dependent):

```sh
cargo run -p c2-harness --bin c2rs -- perf                       # per-obj latency, all fixtures
cargo run -p c2-harness --bin c2rs -- perf-scale --csv docs/perf/perf_scale.csv
python3 scripts/plot_perf.py                                     # regenerate the graph above
```

`perf`/`perf-scale` report a port `Mismatch`/`NotImplemented` per fixture but
only *fail* (non-zero exit) on a capture/replay error or a broken P0.1 replay —
the reference stays the sole judge. The graph is generated by `scripts/plot_perf.py`
(matplotlib — tooling only, outside the std-only Rust workspace).

[wibo]: https://github.com/decompals/wibo
