# DRAFT for `docs/ROADMAP.md` §9.14 — paste verbatim, then delete this file.

Kept out of `ROADMAP.md` on purpose: that file is the recorded add/add conflict
site for concurrent lanes (`docs/rungs/README.md`), the coordinator lands §9.14
serially, and lane `w-eh` is live in `docs/` at the same time. Everything below
is the section text.

---

### 9.14 W-RERANK — three corruptions of one ranking input, and two of them are one defect (2026-08-01)

Lane `w-rerank`, boards **#139**, **#110**, and §9.11's lost suffix. The brief
named three independent corruptions of the emitted board's ranking input and
told this lane to repair all three and re-rank once.

#### 9.14.1 Pre-registration (written and committed before any measurement)

Committed at the lane's first commit, before the base scan was run. `HEAD` is
`99ed418`.

| | registered | refuted if |
|---|---|---|
| **P1** | emitted functions on keys naming `type-ptr` fall from ~8,000 to ≤ 300 | > 1,500 remain |
| **P2** | the census numerator is **unchanged to the unit** — 706,402 bodies, 36,059 emitted — because `mark_whole` is diagnostic-only | any Δ ≠ 0 |
| **P3** | **#110 and #139 are the same defect**: the one operand-type gate removes ≥ 90 % of the `-whole{k≥2}` over-count | the drop is < 5,000 bodies |
| **P4** | some `-more` bodies become measurable once the phantom grant is gone: 0–8,000 | > 30,000 |
| **P5** | §9.11's repair is **total**: every blocked row gets a completeness reading, printed residue named, and agreement with the key's own suffix is 100 % where the key carries one | any row with no reading, or any disagreement |
| **P6** | the mechanized measure-vs-emitter guard **goes red on the base tree** at exactly the ptr class, and green at tip | it passes at base (then it is not a control) |
| **P7** | the re-ranked emitted board moves 2–6 rows in the top 20; ≥ 1 row dies; ≥ 1 row newly appears | no row moves |
| **P8** | the guard, run over the whole correspondence, finds **≥ 1 further disagreement** besides ptr4 | it finds exactly zero others |
| **P9** | `scripts/gate.sh` PASS, 0 mismatch, same verdict count; `c2rs selftest` PASS | any mismatch anywhere |
| **P10** | the differential control for repair 1 — a fixture in the shape the measure was refusing — is accepted by the port and `Port=Match` | the port refuses or mismatches |

**The direction of each corruption, registered before measuring**, because
§9.13's E4 is that the two directions have different controls:

* a measure **narrower** than the emitter manufactures a phantom rung (#139,
  #110) — visible as a key naming a construct that was never a blocker;
* a measure **wider** than the emitter manufactures phantom *completeness* — a
  row that reads takeable and is not. `census/gate disagreement` cannot see
  either, because neither changes what the port accepts.

Registered candidates for P8, named before the guard was written: the
one-byte-unsigned class, the `volatile` qualifier, the `2C` conversion, the `55`
call-end annotation, and the pointer-arithmetic guard.
