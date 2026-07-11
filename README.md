# c2-rs

**Clean-room, I/O-behavioral native port of the MSVC Xbox 360 PPC compiler
backend `c2.dll`, plus the differential test harness that grades it.**

> **STATUS: P0.1 GREEN, port still a stub.** Standalone-c2 IL **replay (P0.1)**
> is **proven** — feeding a captured IL bundle back through `c2.dll` alone (via
> the `c2host` stub under wibo) reproduces the pipeline `.obj`
> **byte-for-byte** on all fixtures, so the differential's **reference side is
> now real**. The native **port** (`PortC2`) remains a **stub** (no compiler
> pass is ported) — that is the open gate. The oracle self-test (determinism +
> IL-capture stability) also still runs green.

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
| `crates/c2-core` | The port itself (**STUB**) — the `Backend` trait, `PortC2` (returns `NotImplemented`), and a `passes/` tree documenting the pass order and first-port targets. |
| `crates/c2-reference` | Drives the **real** `cl.exe`/`c2.dll` under wibo — the oracle. `compile_obj` (normal `/Ox /GS- /c`), `capture_il` (`/Bd /d2nop` early-abort trick), `capture_reference` + `replay` (**P0.1**, byte-exact standalone-c2 replay via the `c2host` stub), and `ReferenceC2` (bundle→obj Backend). |
| `crates/c2-harness` | The benchmark + `c2rs` CLI. `differential()` proves the reference replay is byte-exact then reports the port status, and the live-today `oracle_selftest()`. |
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
cargo run -p c2-harness --bin c2rs -- diff    fixtures/cpp/add3.cpp   # ReferenceReplay=ByteExact, Port=NotImplemented
cargo run -p c2-harness --bin c2rs -- bench
```

The `replay`/`diff` paths additionally need `strace` (keeps the IL bundle) and
`i686-w64-mingw32-gcc` (builds the `c2host` stub); both degrade to a clean
`SKIP` when absent.

[wibo]: https://github.com/decompals/wibo
