# CLAUDE.md — c2-rs

Guidance for AI coding agents (and humans) working in this repo. Mirrors the
parent milohax conventions that apply here.

**c2-rs** — clean-room, **I/O-behavioral** native port of the MSVC Xbox 360 PPC
compiler backend `c2.dll`, plus the differential harness that grades it. This is
il-witness **angle H** (verifier throughput): a faster `c2` prices every loop in
decomp-synth at once. See `README.md`.

## The one correctness rule

**The real `c2` (under wibo) + a byte-exact obj compare is the SOLE judge of the
port.** The criterion is `port(IL) == c2(IL)` byte-exact with the COFF
`TimeDateStamp` (file offset 4..8) zeroed — **I/O-behavioral, not
binary-faithful**. There is no objdiff target for c2.dll itself and we never try
to reproduce its own instruction bytes; the port may use entirely modern code
(AVX, restructured CFGs) so long as its *output obj* matches. Verification is
coverage-bounded differential testing — corpus breadth is load-bearing, and a
green run is sound only on the IL it was tested against, never a total proof.

Do not add "neutrality" / "behavior-preserving" classifiers as gates. The
compiler is the sole judge.

## Commits

- **Commit by default, small and often**, once verified. Never push unless
  asked.
- **No `Co-Authored-By: Claude`** or any AI/agent trailer — human identity only.
- Focused commits, imperative subject lines.
- Peer agent sessions may advance `main` concurrently: re-check `git log` before
  staging, commit with explicit `-- <pathspec>`.
- **Never commit**: secrets/tokens; captured or generated IL (`_CL_*`, `*.il`);
  build artifacts (`*.obj`, `*.o`, `/target`); absolute machine paths
  (`/home/<user>/…` — use `C2RS_*` env / relative-to-repo defaults; toolchain
  location is env-driven by design). Only the fixtures' `.cpp` is tracked.

## Project context (before "cleaning up")

- `dc3` (Dance Central 3) references, the `e:\lazer_build_gmc1` original build
  root, and the XDK build id `16.00.11886.00` are **intentional** — do not
  scrub or genericize them.
- **Sibling-repo layout assumption**: this repo expects `milohax/{c2-rs, wibo,
  dc3-decomp}` as siblings. Toolchain defaults are relative to the repo root
  (`crates/*/../.. = repo root`). Nothing absolute lives in source.

## Hard constraints

- **std only, zero external crates** (no clap, tempfile, or regex — tiny helpers
  are hand-rolled). If a dep looks unavoidable, STOP and discuss.
- Integration tests + the `c2rs` CLI must **degrade cleanly** when the toolchain
  is absent (`SKIP: toolchain absent`) — never panic/fail.
- The native port (`c2-core::PortC2`) is **byte-exact on the MVP function
  class** (straight-line int add-chains, tail calls, a single framed non-leaf
  call) and returns `NotImplemented` outside it — that boundary is the open
  gate, not a fake. The reference seam (`c2-reference::ReferenceC2` /
  `Toolchain::replay`) drives the **real** `c2.dll` under wibo. **P0.1
  (standalone-c2 IL-replay) is PROVEN** — byte-exact on the fixtures. Never fake
  either side: outside the ported class the port must honestly return
  `NotImplemented`, and the oracle is always real c2, never a mock.

## Layout / entry points

- `crates/c2-il` container model, `crates/c2-obj` COFF compare, `crates/c2-core`
  native port + `Backend` trait, `crates/c2-reference` real-toolchain oracle,
  `crates/c2-harness` benchmark + `c2rs` CLI.
- Correctness benchmark = the **oracle self-test** (determinism + capture
  stability): `cargo run -p c2-harness --bin c2rs -- selftest`.
- Performance benchmark (angle H) = `c2rs perf` — IL-bundle→obj latency, the
  in-process port vs standalone `c2.dll` under wibo (both emit the *same* obj;
  the port is confirmed byte-exact before timing). ~200–270× on the fixtures.
- Portable lane (no toolchain): `cargo test --workspace` — unit tests run;
  integration tests skip when `Toolchain::locate()` is `None`.
