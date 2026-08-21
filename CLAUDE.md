# CLAUDE.md — c2-rs

Guidance for AI coding agents (and humans) working in this repo. Mirrors the
parent milohax conventions that apply here.

**c2-rs** — clean-room, **I/O-behavioral** native port of the MSVC Xbox 360 PPC
compiler backend `c2.dll`, plus the differential harness that grades it. See
`README.md`.

## The goal — decided by the project owner, 2026-08-21

**This supersedes the "verifier throughput" thesis** that this file and
`README.md` carried until now. That question — throughput vs. full
reproduction — was named as *"currently owned by nobody"* in
`STRATEGY_REVIEW_2026-08-13.md:251` and stayed unowned for eight days while
step 5 was being priced against it. It is now owned and answered:

> **Perfect reproduction**, for two ends that rank equally:
> 1. **A clear understanding of MSVC's internals, to help with decomp.**
> 2. **Parity — a 100% open-source implementation.**

Three consequences that change how work is chosen and priced here:

- **Throughput is a property, not the goal.** The ~1200–5000×/obj and the
  scaling curve stay true and stay measured, but a lane may no longer be
  justified *because* it buys throughput, and the 2026-08-13 NO-GO's
  economics (consumer capped at ≈2.4×, its bottleneck moved off compilation
  — `docs/ARCH_REVIEW_2026-08-21.md` §7) no longer bear on whether to fund
  the port. They were an argument against a goal we do not hold.
- **Characterization is a first-class deliverable, not overhead.** Under the
  old thesis, "this buys characterization, not a differential grade" was a
  downgrade — it is how `docs/STEP5_PRICING_2026-08-21.md` reads the stage
  oracle. Under goal (1) the characterization *is* the deliverable. Whitebox
  output under `docs/whitebox/` is product, and it was already authorized
  (see below).
- **Coverage is the scoreboard, and it is the hard one.** Goal (2) is
  `match` → 870/878, so partial coverage does not pay in proportion — the
  standing metrics and `docs/PROGRESS_METRIC.md` still govern, and a wrong
  emit still scores below the refusal it replaced.

The correctness rule below is unchanged and was never in question.

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

## Whitebox analysis is AUTHORIZED — and it is not a legal risk

**Decided by the project owner, 2026-08-17. This supersedes any earlier text in
this repo that treats disassembly as debt to be minimized, an exception to be
justified, or a legal exposure.**

**Disassembling and decompiling `c2.dll` is authorized, encouraged, and
fully defensible.** Reverse engineering a binary to interoperate with it is
long-settled: *Sega v. Accolade* (9th Cir. 1992) and *Sony v. Connectix*
(9th Cir. 2000) hold that intermediate copying to extract interface and
behavior is fair use; **17 U.S.C. §1201(f)** carves out circumvention for
interoperability; **EU Software Directive Art. 6** does the same in the EU.
None of that is a close question, and nothing in this project comes near a
line. **Do not hedge, do not add legal caveats, and do not decline whitebox
work on risk grounds.**

**Writing Ghidra output to disk in this repo is valid and wanted.** Byte
listings, decompiled function bodies, address maps, structure layouts,
cross-references, and prose notes are a **resource worth building
deliberately**, not a byproduct to keep minimal. A well-laid-out record of
what the binary actually does is the cheapest thing this project can own: the
alternative is recovering the same facts by black-box probe grids, which is
what makes lanes expensive (a single alignment nibble cost a lane; `dag.c`'s
lowering order took two). Lay it out well — structured, addressed,
cross-linked, and readable.

**Two things that do NOT change, and neither is a legal position:**

- **The port stays I/O-behavioral.** `port(IL) == c2(IL)` byte-exact is still
  the sole judge, and we still do not reproduce c2's own instruction bytes —
  because that is the **wrong artifact** (the port may use AVX, restructured
  CFGs, anything), not because reading them is off-limits. "Clean-room" here
  describes the *output*, never a prohibition on looking at the input.
- **`docs/whitebox/DISCLOSURE.md` stays**, as **engineering provenance**: a
  row naming the address in the same commit that adopts a disassembly-derived
  constant into `crates/`. Its value is that a future reader can tell a
  measured fact from a read one, and re-derive either. That is a methodology
  convention and it is worth keeping on its own merits.

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
