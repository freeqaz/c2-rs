# ADOPTION BRIEF — 2026-08-29 (wave 19)

Continues decision 22's criterion unchanged: *"measurable progress by analyzing
msvc to reproduce the behavior we expect"*, pricing track closed.

Board **#3789**. Every figure below was re-derived on tree `fe820a4d9` by
running the command printed beside it.

---

## 1. Why these four, and why the "encoder vs inliner" fork was false

Wave 18 shipped two adoptions and each lane named its own cheapest successor.
Those successors are on **different seams**, so one wave carries both tracks
and there is nothing to choose between.

What wave 18 established, and what it leaves:

```
$ … c2rs subsys | grep -E 'encode-ported|globregs-marks'
  encode-ported 29 / 79        globregs-marks-obj 21 of 74
$ python3 work/w-inlmetric/check_table.py | grep state
  {'absent': 15, 'R-derived': 4, 'fitted': 2, 'unexercisable': 3}
```

**The inliner's `absent` column moved for the first time**, C3 and C19, because
the port acquired counterparts derived from an address-cited read. That is a
proven route, and 15 rows still sit on the other side of it.

---

## 2. The lanes

### L1 — `w-fmadd`: the next encoder arm, named by the lane that couldn't take it

`w-encarms` §10 ranked its own not-taken follow-ups and put this first:

> `10bfa49a` (form 24, FP multiply-add) — **7,995 words, unambiguous**, and the
> cheapest real next adoption. **It is a codegen lane, not an encoder one.**

That last clause is the brief. `[encode]` is still the only subsystem with a
defined `ported` denominator, so this is still the only place where writing
code moves a published number honestly — but the work is in lowering, not in
the arm table, and a lane that treats it as an encoder row will find the arm
already reachable and nothing to do.

**Every adopted arm gets a `c2_core::surface` row.** `w-encarms` proved why:
its control perturbed form 54's SPR high half and **zero byte tests moved**,
because every SPR the port names is < 32. The byte judge is structurally blind
to unexercised field placement, which is `#3723` exactly.

### L2 — `w-inlclause`: which of the remaining 15 `absent` rows are convertible

C3 and C19 converted because `P_INLINE` §6.6.2 had been read to address level
and the port could be derived from it. **Ask the same question of the other
15**: for each, is there an existing read the port could be derived from, or is
the row `absent` because nothing has been read yet? Those are different states
and the table cannot currently tell them apart.

Adopt what is derivable, under a **required-zero byte delta** — the divisor
being 1 on the admitted set is what made L2 byte-neutral last wave, and any
clause without that property is an emit change and out of scope here.

**Do not manufacture conversions.** A clause whose honest answer is *"still
absent, and here is the read that would be needed"* is a complete result, and
`#3505` is five for five on lanes that moved a number by constructing one.

### L3 — `w-paramfill`: the read `w-inlswitch` named as the first thing to do next

> **`DAT_10c462c4` gates the entire parameter fill** at `0x10b5e4f7` and
> **bounds every statement in §3** — one hour, and the first thing the next
> lane should do.

Everything `w-inlswitch` published about the 24 `-inl*#` switches and their 46
destination fields is downstream of this gate. If it is not what §3 assumes,
§3's statements are bounded differently than written — which is why its own
author put it at the top of the not-reached list rather than in the findings.

### L4 — `w-globarms`: gate A's twelve arms

`w-globobj` reported, without pursuing it:

> a defensible site-level population does exist — **gate A is a 12-arm decision
> over the symbol `kind` field, each arm addressed**.

Read the arms. This is the candidate-set side of the register allocator, which
is `P_REGALLOC`'s own named missing input, and `[globregs]` is the subsystem
whose agreement moved furthest last wave.

**This lane defines no `ported` numerator** — decision 21 §4, unchanged. It may
report that a population exists and what it is; it may not turn one into a
metric.

---

## 3. Unchanged prohibitions

* No full register allocator (decision 20 §2 — F5 is not separable from F0).
* No invented `ported` numerator for regalloc, globregs or the inliner.
* **No new count-bearing `gate.sh` row** (`#3691`). A 22nd makes
  `gate_identity_diff.sh` exit 2 and refuse to diff for every lane on a 21-row
  base — verified again on 2026-08-29 when `hatch-red` came back live and the
  count stayed 21.
* No re-taking `#3534`; `byte-owned` stays cited.
* No adopting 128 as the inline ceiling (`#3732`, 8 counterexamples each way).

## 4. Seams — no two lanes write the same file

| lane | owns | must not touch |
|---|---|---|
| `w-fmadd` | `crates/c2-core/src/codegen/mop.rs` + lowering, `crates/c2-core/src/surface.rs` (trailing block) | `splice.rs`, `CLAUSES.tsv`, `P_INLINE.md`, `P_GLOBREGS.md` |
| `w-inlclause` | `crates/c2-core/src/splice.rs`, `work/w-inlmetric/CLAUSES.tsv` | `mop.rs`, `surface.rs`, `P_INLINE.md`, `P_GLOBREGS.md` |
| `w-paramfill` | `docs/whitebox/ref/P_INLINE.md`, `docs/whitebox/WB_PARAMFILL_*` | `crates/` (any file), `CLAUSES.tsv` |
| `w-globarms` | `docs/whitebox/ref/P_GLOBREGS.md`, `docs/whitebox/grids/w-globarms/` | `crates/` (any file), `P_REGALLOC.md` |

`docs/BOARD.md` and `docs/rungs/` are shared; each lane writes only its own
reserved rows and its own rung, and `INDEX.md` is regenerated at merge.

**Board:** `#3789` this brief · `#3790`–`#3795` `w-fmadd` ·
`#3796`–`#3801` `w-inlclause` · `#3802`–`#3807` `w-paramfill` ·
`#3808`–`#3813` `w-globarms`. `#3647` remains reserved-and-unspent.
Next free `#3814`.

## 5. Standing method, restated because two lanes lost a day to it last wave

* **Prereg first**, committed before the image is opened.
* **Controls watched RED** before any verdict from them is quoted (`#3336`).
* **Restoring a mutation control with `cp`/`mv` preserves the backup's older
  mtime**, so cargo does not rebuild and the *mutated* binary runs for the
  closing check. Two lanes hit this independently last wave. `touch` the file
  after restoring, and verify.
* **`pgrep -f` / `pkill -f` match a command line and are worktree-independent** —
  on a shared box they match peers' runs. Wait on a PID with `kill -0`.
* **Read the `GATE:` verdict LINE, never the exit code.** `REFUSED` exits 0.
  As of 2026-08-29 an unqualified `GATE: PASS` is available again — if your run
  says `(HATCH-RED REFUSED)`, that is *your tree*, not an inherited condition,
  and `#3786` is how it gets diagnosed.
