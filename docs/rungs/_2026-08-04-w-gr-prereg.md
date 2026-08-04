# PREREG — lane `w-gr`, task #40: the `/GR` axis and the RTTI shapes

Registered **before** the first edit to `scripts/`, `fixtures/` or
`crates/c2-harness/tests/`. Tree at registration: `46cec43` (master), branch
`wt-w-gr`. Everything below §1 is recon on the *unmodified* tree, so it cannot
have been tuned to the predictions in §2.

---

## §1 Recon on the unmodified tree — measured, not assumed

All of it with `scripts/gt_capture.sh` (real `cl.exe` under wibo), 2026-08-04,
tree `46cec43`.

### 1.1 `/GR` is **not** the default for this compiler

The single most load-bearing fact for this lane, and the one I expected to go
the other way (MSVC's documented host default is `/GR` **on**):

| source | `/O1 /Oi /EHsc` | `… /GR` | `… /GR-` |
|---|---|---|---|
| `struct B{virtual ~B();virtual int f();int b;}; B::~B(){} int B::f(){…}` | `.text .rdata .text .pdata .text` | `.text .rdata .rdata$r .data .rdata$r .rdata$r .rdata$r .text .pdata .text` | same as the first column |

So the generated corpus is **structurally unable** to produce `.rdata$r` at any
profile any instrument currently runs, *regardless of what source it contains*.
This is not a source-shape gap alone and it is not a flag gap alone: it needs
**both** halves, which is why task #40 has two.

### 1.2 What actually mints `.rdata$r` — the vftable, not the cast

Grid of 15 shapes × `{/O1 /Oi /EHsc, … /GR}` (`work/w-gr/g1/`). Non-boilerplate
sections only:

| shape | `.rdata$r` under `/GR`? |
|---|---|
| polymorphic class, **destructor defined here** | **YES** (4 records + `.rdata` vftable + `.data`) |
| polymorphic class, **pure virtual + destructor defined here** | **YES** |
| polymorphic class, only `int B::f(){…}` defined (no ctor/dtor) | no |
| base+derived, both `f()` defined, no ctor/dtor | no |
| `dynamic_cast<D*>(p)` | **no** — and the obj is content-identical with and without `/GR` |
| `dynamic_cast<D&>`, `dynamic_cast<void*>` | no |
| `typeid(*p)` returning `const type_info&` | no |
| multiple inheritance, virtual inheritance (no ctor/dtor) | no |
| non-polymorphic control | no |

**The trigger is vftable emission, and vftable emission is driven by a
constructor or destructor body being generated in this TU** — the ctor/dtor is
what writes the vfptr. `dynamic_cast` and `typeid` reference `??_R0` type
descriptors, which land in **`.data`**, not `.rdata$r`; they add nothing on this
axis. A fragment written from the obvious mental model ("RTTI means
`dynamic_cast` and `typeid`") would have produced **zero** `.rdata$r` cases and
read as success.

### 1.3 The `/GR` delta is content-invariant across modes and position-variant

3 fixtures × 8 profiles, `/GR` on each (`work/w-gr/grorder.py`):

* the **`.rdata$r` section contents are byte-identical across all 8 modes** — 1
  distinct content set per fixture, every time;
* their **position in the section table is not**. `/Ox /GR` and `/Od /GR` put
  the RTTI block after 2 sections (packed `.text`); `/O1 /GR`, `/O2 /GR` and
  `/Ox /Gy /GR` after 5 (COMDAT-per-function); `/O1 /Oi /EHsc /GR` after 12,
  with EH `.rdata` records interleaved around it; `/Od /EHsc /GR` places a
  192-byte EH `.rdata` **before** the 8-byte vftable `.rdata`.

Since section index drives symbol index drives `.pdata` association number, the
placement is the part the port has to be right about, and it is a genuine
**cross term** with the code-shape configuration.

### 1.4 `/GR` over the 245 tracked fixtures

`work/w-gr/grdiff.py`, 8 base configurations × 245 fixtures × 2 captures =
**3,920 captures, 27.9 s wall at 12 jobs**, `capture_fail=0`:

* **245 of 245 objs differ bytewise** under `/GR` at every base config — because
  `/GR` is recorded in `.drectve` and `.XBLD$W:C1/:C2`. **That count
  discriminates nothing and I am recording it so nobody quotes it as coverage.**
* dropping `.drectve`, `.debug$S`, `.XBLD$W` and comparing section data +
  symbol count: **3 of 245** differ, at *every one* of the 8 base
  configurations, and they are the same 3 —
  `w14_dtor_delegate_neg.cpp`, `w19_ctor_this_neg.cpp`, `wec_ctor_base_neg.cpp`.
* those same 3 are exactly the fixtures that carry `.rdata$r` under `/GR`.

So the fixture corpus *can already* produce `.rdata$r` — on 3 of 245 files —
and has never once been compiled at a flag string that would show it.

### 1.5 The instrument's own profile is not the workload's

`scripts/sweep_shapes.py` pass B defaults to `--flags "/O1 /Oi /EHsc"` and its
`--help` calls that *"the dc3 workload's own"*. `work/dc3-workload/flags.txt`
reads `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …`. **It is short by
`/GR`, which is the one flag that mints the one section the report exists to
report as missing.** Correcting that default is part of this lane.

### 1.6 Baselines on the unmodified tree

| | |
|---|---|
| `sweep_shapes.py` corpus | **57 fragments, 16,164 cases**, 0 of 56 markers at zero, `SHAPE-CHECK: PASS` |
| workload census (`work/w-bss/census/sections.jsonl`) | 871 objs, 14 distinct names; `.rdata$r` = **24,163 sections over 676 objs** |
| workload names the corpus cannot produce | `.XBLD$W:C1`, `.XBLD$W:C2` (both the census's spelling of the corpus's truncated `.XBLD$W` — a reader artefact) and **`.rdata$r`** |
| lane registry | 12 lanes, `EXPECTED_LANES = 12` |

---

## §2 The predictions

Each is falsifiable, and each says what would make me decline.

**P1 — `.rdata$r` becomes producible, and only with both halves.** After the
fragment lands, `sweep_shapes.py --objs … --flags "/O1 /Oi /EHsc"` (no `/GR`)
will **still** list `.rdata$r` as unproducible; with `/GR` in the profile it will
not. Predicted final "cannot produce" list: **`.XBLD$W:C1`, `.XBLD$W:C2` only** —
the honest remainder goes 1 → **0**.
*Refuted if* the `/GR` run still lists `.rdata$r`.

**P2 — every new case is GRADED; the ungraded baseline stays 96.** The sweep's
ungraded count is cases the *reference* rejects (board #281). Everything in §1.2
compiled clean at both `/Ox /GS- /c` and `/O1 /Oi /EHsc /GS- /c` except
`typeid(a)==typeid(b)`, which needs a real `<typeinfo>` (`error C2676`) and which
I will therefore not write. Predicted ungraded delta: **0**.
*Refuted if* `C2RS_SWEEP_MAX_UNGRADED=96` has to be raised.

**P3 — mismatches: predicted 0, but I put the subjective odds of ≥1 at ~35 %,
and a mismatch is this lane's most valuable possible output.** The sweep grades
at `/Ox /GS- /c`, where `/GR` is off, so the new cases reduce to *plain
polymorphic classes with an emitted vftable* — a `.rdata` COMDAT holding
function pointers, in a section name the writer **already has**
(`PORT_WRITER_SECTIONS`). That is board #276's exact shape: a name the writer
can spell over content it cannot produce. If it fires I report it immediately
and **do not fix it** — this lane does not touch `crates/c2-core`.

**P4 — the payoff metric will not move, and I am saying so in advance rather
than after.** TU match stays **8/878**. Factor **A** binds at 28/871 and `/GR`
does not touch it. Factor **C** stays **169** and the FRONTIER stays **19**:
closing `.rdata$r` in the *corpus* is not adding it to the *writer*, and only
`PORT_WRITER_SECTIONS` moves C. Anyone reading "the `.rdata$r` gap is closed"
as "C is 590 now" has read it backwards.
*Refuted if* any of those four numbers moves.

**P5 — the lane set is 4 new lanes, not 12.** Two base configurations get a
`/GR` twin, each crossed with `/EHsc` (the registry's test requires the cross and
requires the two sides to be equal in count, so lanes come in pairs):

| new lane | why |
|---|---|
| `O1-Oi-GR` `/O1 /Oi /GR` | the workload's own profile, minus EH |
| `O1-Oi-EHsc-GR` `/O1 /Oi /EHsc /GR` | **the dc3 workload's literal flag string.** The `/EHsc` hole (#263) one flag over |
| `Ox-GR` `/Ox /GR` | the *packed* placement class, and the profile `c2rs diff` and `expr_sweep.sh` hardcode |
| `Ox-EHsc-GR` `/Ox /EHsc /GR` | packed × EH — §1.3 shows EH records move the RTTI block |

**What I am deliberately NOT covering, with the measurement behind each:**

* **`/O2 /GR` and `/Ox /Gy /GR`** — §1.3: on all 3 RTTI-bearing fixtures these
  produce the *same section-name order* as `/O1 /Oi /GR`, differing only in
  `.text` sizes, which the plain `/O2` and `/Ox /Gy` lanes already grade. They
  are the same placement class.
* **`/Od /GR`** — same placement class as `/Ox /GR` (packed). `/Od` grades ~1 of
  197 in class and is the fail-closed boundary lane; its `/GR` twin re-grades
  the same refusal.
* **`/O1 /GR` without `/Oi`** — §1.3: identical section list to `/O1 /Oi /GR`
  on every RTTI fixture. The workload passes `/Oi`; the twin without it is the
  weaker of the two.
* **`/GR-` explicitly** — measured identical to the default (§1.1), so it is a
  spelling, not an axis.

This is a real cost decision and not only a taste one: `scripts/mode_cross.sh`
costs ~5 m 45 s cold over 12 lanes, so 12 new lanes would roughly double the
merge gate's most expensive row to buy 6 copies of one content-invariant delta.
4 lanes is +33 %.

*I would decline the twins beyond `O1-Oi-EHsc-GR` if* the `/GR` lanes graded
**0** corpus cases differently in content after the fragment lands — that would
make them the `/O1 /Gy` situation (a flag that is not being varied) rather than
the `/O1 /EHsc` situation (a different obj, same verdict).

**P6 — the fragment.** 250–600 cases, one file
`scripts/sweep.d/91-rtti-vftable.py`. Predicted new markers: `dynamic_cast`,
`typeid`, `virtual inheritance`, `vftable-emitting ctor/dtor` — added to
`sweep_shapes.py`'s table **in the same commit** as the cases that make them
non-zero, so `--check`'s zero baseline never has to be raised.

**P7 — the counterfactual will separate.** Reverting a guard in
`crates/c2-core` (not shipped; reverted in the same script) will produce a
**non-zero** mismatch count concentrated in the new fragment, and the run will
prove `git status --porcelain crates/` empty afterwards. A breaker that lights
up every fragment equally proves nothing and I will say so if that is what I
get.
