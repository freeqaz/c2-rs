# w-decouple — PREREG

Frozen **before the first `crates/` change of any kind**, including the two
counterfactual builds §3 is written about. Scored in the rung's §8.

    Lane:    w-decouple, worktree branch `worktree-agent-ad5efaf54aa302999`
    Base:    master `1326c86f` (the `w-wordwrap` merge). Merge-base checked.
    Binary:  `work/w-decouple/c2rs-base`, md5 `2b866cf73559305d3ccde85bb3692fb4`,
             built at the merge-base and KEPT (#2409 — `git checkout master --
             crates/` is not a counterfactual; #2512 — `git checkout <rev> --
             path` STAGES).
    Stamp:   dc3 `a8cb9ca639df2e938553ae24200307fa7a31abce`, tracked tree clean
             (0 dirty lines, untracked-files=no). **878** lines in
             `work/dc3-workload/files.txt`, `wc -l`-checked, USED AS IS and
             NEVER regenerated (#2700). Toolchain
             `compilers/X360/16.00.11886.00`; wibo `../wibo/build/release/wibo`.
             **363** fixtures at freeze (`ls fixtures/cpp/*.cpp | wc -l`).
    Base scan (this lane's own, run before any change):
             match **23** · mismatch **0** · codegen-gap **0** · vocab-gap
             **848** · port-error **0** · capture-fail **7** · FRONTIER **4** ·
             `fnbyte-exact` **35,810** · `fnbyte-differs` **1,898** ·
             `fnbyte-partial` **10** · `fnbyte-refused` **114,622** ·
             factor A/B/C/D/E **28 / 338 / 169 / 23 / 2**.

---

## 0. Declared prior — what is already on disk, and is NOT scored

Unscored on purpose: `w-pool` scored 30/31 and correctly called that a
calibration failure because its claims restated the tree.

* `w-front5` #2621/#2624: `src/Main.cpp` stops at `main` (4 B); `mmio.cpp`
  binds 4 of 11 and stops at `mmioSeek` (8 B). **Re-derived at this base**,
  from this lane's own captures at the workload's own flags —
  `work/w-decouple/glwalk2.py` on `il/main` and `il/mmio`. Both hold, byte for
  byte, on a dc3 that has moved since (`d7a3c1aa` → `a8cb9ca6`).
* `w-front5` #2622/#2623: the naive one-clause widening binds 2 of 15,
  converts 0, and costs **−1 `fnbyte-exact`**, all of it on
  `src/system/synth_xbox/FFT.cpp`. **NOT re-run** — the brief forbids it and
  the measurement stands.
* `w-main` P12: `Main.cpp`'s reference obj carries a second code region
  (`__unwind$2585` at `.text+0x54`), **two** `.pdata` and a 64 B `.rdata`
  (`__ehfuncinfo$main`, `__unwindtable$main`, `$T2592`). Re-confirmed byte for
  byte off `work/w-decouple/ref/main.dump`.

## 0b. Findings already in hand at freeze — recorded, NOT scored

Registered here so they cannot be presented later as predictions.

* **The brief's own item-8 instrument is wrong on both target TUs.**
  `emit-bound == emit-gate-segments` reads **1 == 1** on `src/Main.cpp` and
  **11 == 11** on `mmio.cpp`, and **neither binds**. The field that answers
  `CEILING.md` §11.4 item 8 is `gate_cause` / `gate_causes`, exactly as that
  item says — `emit-*` is `EmitBinding`, a THIRD binding (#918).
* **`mmioClose` calls `mmioFlush`, which this TU DEFINES** (`work/w-decouple/
  ref/mmio.dump`, `.text #14` `REL24 -> [33] mmioFlush`). So mmio has an
  intra-TU call edge and the inline fence is live on it the moment it binds.
  No published price for mmio names this.
* Every unclaimed mangled `.gl` run on both TUs is an **undefined external**
  in the reference obj that a **blocked** body calls: `??0App@@QAA@HPAPAD@Z`,
  `??1App@@QAA@XZ`, `?Run@App@@QAAXXZ` (all `main`'s) and
  `?FreeHandle@@YAXPAX@Z` (`mmioClose`'s).

---

## 1. THE DESIGN THIS LANE WILL BUILD

`gl_defined_names` is read at **three** call sites, not the two #2623 names:

| site | what it asks | this lane |
|---|---|---|
| `bind::Bindings::per_record` | *"what name does each defined body carry, so the writer can emit its symbol?"* | **WIDE** — admits a record name that fits the 8-byte COFF inline field |
| `bind::defined_name_set` | the **census**'s inline-fence ground set (`Bindings::positional` has only mangled names) | **NARROW — unchanged, byte for byte** |
| `gl::plain_external_defined_names` | the **gate**'s fence EXEMPTION (W-FENCE2) | **NARROW — unchanged, byte for byte** |

plus `diag::decode_causes`, which transcribes the gate binding and must follow
it or the published cause stops naming the real gate.

**The claim being tested is that these were coupled by IMPLEMENTATION and not
by SPECIFICATION.** The widening is *monotone*: it can only change a TU on
which the incumbent walk returned the EMPTY pair. On exactly those TUs
`defined_name_set` and `plain_external_defined_names` are `∅` today and stay
`∅`, so the fence's behaviour is unchanged **everywhere**, and on a
newly-binding TU the gate's own fence gets STRONGER (`defined` = the full name
list, `exempt` = `∅`).

**The residue this ships, named up front**: on a newly-binding TU with an
intra-TU call edge, `exempt` is `∅`, so the gate refuses wholesale at
`locally-defined-callee` where W-FENCE2's exemption would have handed the TU to
`comdat::fenced_inlined_callee`. `mmio.cpp` is such a TU. §3 sizes it.

---

## 2. PREDICTIONS

### The decoupling

| # | p | claim |
|---|---|---|
| **P1** | **0.88** | The two sets separate. `defined_name_set(gl)` and `plain_external_defined_names(gl)` return **bit-identical** sets on all 878 workload TUs and all 363 fixtures under the shipped build — verified as an assertion in code, not inferred from a metric |
| **P2** | **0.90** | **THE DECIDING ROW. Conditional on P5 (both TUs bind): `fnbyte-exact` delta is ≥ 0, and specifically EXACTLY 0 — 35,810 → 35,810.** This is the row `w-front5` lost at −1. If P5 fails this prediction is VOID, not hit |
| P3 | 0.85 | `src/system/synth_xbox/FFT.cpp` — which carried 100 % of #2622's loss — is bit-identical in every `emit` key and in `fn_in_class` |
| P4 | 0.92 | `mismatch` stays **0**: 878 TUs, and 341+ fixtures at **both** `/O1` and `/Ox` |
| **P5** | **0.80** | Both `src/Main.cpp` and `src/xdk/nuispeech/mmio.cpp` BIND — `gl-stop-name-not-mangled` leaves both `gate_causes` sets |
| P6 | 0.60 | Exactly **2** of the 15 `gl-stop-name-not-mangled` TUs bind (#2627's reach, reproduced on a decoupled build) |
| P7 | 0.75 | `unclaimed-gl-symbol` appears on both TUs' `gate_causes` after binding (w-front5 §5.3) **and is not an independent mechanism**: every unclaimed run is an undefined external a blocked body calls, so it discharges with that body |
| P8 | 0.70 | `mmio.cpp` gains **`locally-defined-callee`** in `gate_causes` — the `mmioClose → mmioFlush` edge, which no published price for this TU names |

### The fixtures

| # | p | claim |
|---|---|---|
| P9 | 0.75 | A fixture in `w-vsnprnc`'s `ecshort` shape — `extern "C"`, defined-function names ≤ 8 bytes, in-class body — grades **`vocab-gap` at base** and **`match` at tip**, at `/O1`. First obj this project has graded for a DEFINED symbol using the 8-byte inline name field (#2374 measured the refusal and never the accept) |
| P10 | 0.60 | The same at `/Ox` |
| P11 | 0.65 | Its `_neg` cell's must-fail mutation deletes the whole conjunction and the cell goes red (#2698/#2699) |

### The conversions — conditional on the decoupling, declared as such

| # | p | claim |
|---|---|---|
| P12 | **0.05** | **Conditional on P5**: `mmio.cpp` converts (match 23 → 24). Priced against: `mmioClose`'s 124 B `cflow-if-n` body (w-ifn's six), plus the `mmioFlush` fence |
| P13 | **0.01** | **Conditional on P5**: `Main.cpp` converts. Priced against: 13 EH mechanisms + a second code region + a second `.pdata` + the 64 B `.rdata` EH set |
| P14 | 0.90 | `match` stays **23** and FRONTIER stays **4** |

### Neutrality and gate

| # | p | claim |
|---|---|---|
| P15 | 0.85 | **L1** `class`, 878 by NAME (never basename — #2667): **0 moved** |
| P16 | 0.80 | **L2** `gate_cause`/`gate_causes` as SETS, 878 by name: moves on **exactly 2** rows, both directional — `−gl-stop-name-not-mangled` |
| P17 | 0.85 | **L3** `emit[fnbyte-exact/-differs/-refused]` per-TU byte TRIPLES, 878 by name: **0 moved** |
| P18 | 0.70 | `gap-metric` key SET unchanged (0 vanished, 0 appeared); the keys whose VALUE moves are only the `decode-cause` histogram rows for the two TUs |
| P19 | 0.75 | Factors A/B/C/D/E unmoved at 28/338/169/23/2 |
| P20 | 0.90 | Full gate 18/18 PASS, **0 mismatch anywhere**; `cargo test --workspace --release --no-fail-fast` green; `c2rs selftest` 0 ERROR |
| P21 | 0.55 | At least one existing `c2-il` unit test pins the incumbent behaviour and must be re-pointed (not deleted) |

### The registered counterfactual — NOT shipped

| # | p | claim |
|---|---|---|
| P22 | 0.55 | Build **B** (identical to the ship, except `plain_external_defined_names` ALSO takes the wide walk) also holds `fnbyte-exact` at 35,810 and `mismatch` at 0, and differs from the ship on **exactly one** row — `mmio.cpp` losing `locally-defined-callee` |
| P23 | 0.35 | Build B converts nothing either |

---

## 3. WHAT WOULD FALSIFY THE LANE

* Any `mismatch` anywhere. The widening admits a name class the WRITER's
  `emit_symbol` handles (`name.len() <= 8` → `b.name8`) but which no capture
  has graded on a **defined function symbol** — `memcpy` grades the inline path
  for an *undefined external* only. P9 exists to close that.
* `fnbyte-exact` < 35,810. Then the sets do NOT separate and the architectural
  answer is negative — which is a real result and will be reported as one.
* Both TUs failing to bind. Then the design is wrong about which walk the gate
  runs, and everything downstream is void.

## 4. WHAT THIS LANE WILL NOT DO

* Re-run #2622's naive widening. Measured; the brief forbids it.
* Convert either TU by force. `mmio.cpp` needs `mmioClose` and `Main.cpp` needs
  an EH layer; neither is a lane's remainder.
* Regenerate `work/dc3-workload/files.txt` or the flags (#2700), or the fixture
  list before the last fixture is authored.
