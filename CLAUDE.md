# CLAUDE.md — c2-rs

Guidance for AI coding agents (and humans) working in this repo. Mirrors the
parent milohax conventions that apply here.

**c2-rs** — clean-room, **I/O-behavioral** native port of the MSVC Xbox 360 PPC
compiler backend `c2.dll`, plus the differential harness that grades it. The
thesis is **verifier throughput**: a faster `c2` speeds up every
compile-in-the-loop decompilation workflow at once. See `README.md`.

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
compiler is the sole judge. (This bans a semantic classifier standing in for
the byte judge; it does **not** ban neutrality *measurement* — comparing
per-fixture verdicts against real c2 at both modes is required, and caught
live wrong emit #2533.)

A wrong emit scores strictly below the refusal it replaced — a scoring rule
(`docs/PROGRESS_METRIC.md`), unchanged. It is **not** a licence to refuse
without pricing: every new fence is priced **two-sided** (#1042, NC-5/#2691 —
both times the refusal's own cost was counted, the answer flipped), in the
units the goal is written in, before it ships.

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
- **Toolchain resolution is self-contained**: compilers come from
  `./compilers/` (populated by `scripts/fetch_compilers.sh`, gitignored — MS
  binaries are never committed), wibo from `PATH` or a sibling `../wibo`
  build; sibling `../dc3-decomp/build/compilers` is only a compat fallback.
  Defaults are relative to the repo root (`crates/*/../.. = repo root`).
  Nothing absolute lives in source.

## Hard constraints

- **std only, zero external crates** (no clap, tempfile, or regex — tiny helpers
  are hand-rolled). If a dep looks unavoidable, STOP and discuss. The rule binds
  `crates/`; it is never a reason to move an instrument that grades the port out
  of the workspace — anything whose output is quoted as evidence must run under
  `cargo test` or `scripts/gate.sh` (#1406).
- Integration tests + the `c2rs` CLI must **degrade cleanly** when the toolchain
  is absent (`SKIP: toolchain absent`) — never panic/fail.
- The native port (`c2-core::PortC2`) is **byte-exact on the function classes
  its fixtures fence** and returns `NotImplemented` outside them — that
  boundary is the open gate, not a fake. On the 878-TU workload the admitted
  classes are bimodal: ten one-function classes at 11/11 and five call-bearing
  classes at 0.000 over 1,106 bodies (`ROADMAP.md` §10.30) — do not quote "the
  MVP class" as if it covered the workload. The reference seam (`c2-reference::ReferenceC2` /
  `Toolchain::replay`) drives the **real** `c2.dll` under wibo. **P0.1
  (standalone-c2 IL-replay) is PROVEN** — byte-exact on the fixtures. Never fake
  either side: outside the ported class the port must honestly return
  `NotImplemented`, and the oracle is always real c2, never a mock.

## Orient before you measure

- **`docs/STATUS.md` is the one-page answer to "where is this project".** Its
  metric block is generated — regenerate with `scripts/status.sh --write`, never
  hand-edit. It also carries the *traps*: which numbers are targets, which are
  drivers, and why `mismatch 0` is not evidence of correctness. Read it before
  quoting a number out of `ROADMAP.md`, which is 8k lines of session history and
  contains many superseded snapshots.
- **`docs/BOARD.md` enumerates the numbered items** (`#1`…) that `ROADMAP.md`
  references everywhere but never lists. New items take the next free number and
  are added there in the same commit.

## Units of work

Three lane kinds are first-class (`docs/rungs/README.md` § "Lane kinds"):

- **Fixture-claim rung** — the default: names fixtures, moves the census, may
  convert a TU. The right unit for TU-shaped work only.
- **Construct rung** — builds shared machinery (IR, passes, gate predicates)
  by re-expressing already-byte-exact classes through it. `Fixtures: none`,
  `Census: +0`, **required-zero byte delta**, graded by an identity diff of
  per-lane gate counts (board #290's pattern).
- **Characterization lane** — reads real-c2 behavior (whitebox + obj grids)
  and lands address-cited findings under prereg; predicted reach 0
  (`wb-live`'s pattern).

**Phase work (CEILING §6.1) is dispatched as construct rungs and
characterization lanes, never as TU lanes** — a TU lane cannot carry a phase,
and forcing one to produced 150 rungs of predicted saturation. Every lane
reports one outcome in its rung header: `converted`, `declined`, `instrument`,
`built`, or `FAILED` — a lane that produced none of its deliverable says
**FAILED** in those words, not a compound headline.

## Layout / entry points

- `crates/c2-il` container model, `crates/c2-obj` COFF compare, `crates/c2-core`
  native port + `Backend` trait, `crates/c2-reference` real-toolchain oracle,
  `crates/c2-harness` benchmark + `c2rs` CLI.
- Correctness benchmark = the **oracle self-test** (determinism + capture
  stability): `cargo run -p c2-harness --bin c2rs -- selftest`.
- Performance benchmark (angle H) = `c2rs perf` (per-obj latency) and
  `c2rs perf-scale` (throughput vs concurrency) — the in-process port vs
  standalone `c2.dll` under wibo (both emit the *same* obj; the port is confirmed
  byte-exact before timing). ~1200–5000× per obj; the port scales across cores
  (~922k obj/s on one thread, ~15.3M at 32) while c2 saturates (~2.9k). Graph in
  the README is
  regenerated by `scripts/plot_perf.py` (matplotlib — tooling, outside the
  std-only workspace); data via `perf-scale --csv docs/perf/perf_scale.csv`.
- Portable lane (no toolchain): `cargo test --workspace` — unit tests run;
  integration tests skip when `Toolchain::locate()` is `None`.
