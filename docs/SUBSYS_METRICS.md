# SUBSYS METRICS — the per-subsystem scoreboard

> **GENERATED — do not hand-edit.** Regenerate with
> `scripts/subsys_metrics.sh --write`. Tree `598b646c2` (DIRTY), generated
> `2026-08-29T03:15:09Z`. Every number below is re-verified against this tree by
> `cargo test -p c2-harness --lib subsys`, which `scripts/gate.sh`'s
> unit row runs; the seven positive controls run beside it, plus the two
> checks that are not fabrications (#3665, and the observer exclusion).


**Status: adopted 2026-08-26 (lane `w-submetric`, boards `#3617`–`#3622`).**
Funded by [`DECISIONS_2026-08-22.md`](DECISIONS_2026-08-22.md) § Decision 15,
the owner's restructuring of the working goal: *"the overall TU goal is too
broad because it is binary. we need a smarter goal … focus on building tools
we can use to measure our progress for each unit."*

One 4-tuple per [`whitebox/ref/SUBSYS.md`](whitebox/ref/SUBSYS.md) §1
subsystem — **read**, **agreement**, **exercised**, **byte-owned** — with
**every denominator printed beside its numerator**.

## 0. The separation rule (read this even if you read nothing else)

> **These keys are PROGRESS instruments and never correctness criteria.**
> The real `c2` under wibo plus the byte-exact whole-obj compare is the SOLE
> judge of the port (`CLAUDE.md`). A `subsys-metric` row going green while
> `mismatch` reads 1 is a FAILING tree.

[`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md) §0 is the standing
template for every gradient added after FBM, and this one adopts all five
properties verbatim, as `decode-reach-*` and `symbind-*` did before it:

* **Never in `scripts/gate.sh`'s verdict**, and it must never be added
there. It does not print inside a `c2rs gap` scan at all — it is a
separate offline subcommand, so it cannot move the gate's 21-row count
table even by accident.
* **Its own block**, under its own disclaimer, apart from the class table
that carries `match`/`mismatch`.
* **Namespaced keys** — `subsys-metric <key> <value>`. No existing key,
predicate or denominator is narrowed, widened or redefined here.
* **It licenses no emit.** A subsystem row going green is not a reason to
accept a shape or to widen the admitted set.
* **Unrepresentable over an empty scan** — a strength with no data prints a
**named residue**, never `0`, never silence.

## 1. The three standing traps, verbatim

1. **THE SIGNAL IS THE CHANGE IN EACH STRENGTH, NEVER ITS DISTANCE FROM 0 OR 100. A subsystem can go from 20 % to 90 % understood with `match` unchanged; that movement is what this table exists to make visible, and a row's absolute height is not a grade.**

2. **A GREEN ROW IS A STATEMENT ABOUT THE POPULATION THE INSTRUMENT CAN REACH. Every denominator here says which tree and which enumeration it came from, because the same subsystem has more than one defensible denominator: the band and the TU-level attribution differ by up to 3.8x (inliner, 93 vs 350). A ratio without its denominator's basis is not a reading.**

3. **THESE KEYS LICENSE NO EMIT. They are progress instruments under docs/FUNCTION_BYTE_MATCH.md §0 — never in scripts/gate.sh's verdict, their own block under their own disclaimer, namespaced, NO-RESULT rather than a ratio over zero. The sole judge of the port is real c2.dll under wibo plus a byte-exact obj compare, and a wrong emit still scores strictly below the refusal it replaced.**

## 2. What each strength actually is here

| strength | this instrument's answer |
|---|---|
| **1 read** | a **containment, never a ratio**: `sites ⊇ read ⊇ ported`. `sites` is the subsystem's enumerable population, **recomputed from `FUNCS.tsv` on this tree** where it is a band; `read` is what the `P_*.md` page says it read, in the page's own unit; `ported` is **measured on two rows and a named residue on the other eight** — see §4, and note that each measured row's `ported` is in its OWN unit (encode arms, `.gl` arms), so the containment nests SITE SETS and the three counts' RATIOS must not be compared |
| **2 agreement** | the page's own **evidence-mark census** — `[O]` obj-confirmed against `[R]`+`[O]`+`[I]` — plus, where a page carries a real differential, that differential quoted with its own denominator. **A mark is a page annotation, not a site.** Two rows carry more: `encode` has a measured differential, `inline`'s is being built by lane `w-inlmetric` and prints `PENDING` |
| **3 exercised** | a **labelled workload-output proxy** where one exists, from the committed real-`c2` section census of the workload; a named residue otherwise. **Per-SITE exercise is unmeasurable on this tree for all ten** — nothing traces `c2.dll`'s own addresses over the workload, so no row can say which of its functions the workload entered |
| **4 byte-owned** | **CITED, NEVER RE-MEASURED.** Board `#3534` measured it 2026-08-25. Decision 15 says so in its own words; re-funding that read is what this repo calls *"check the board before dispatching"* |

**The mark census's honest limit, stated before its numbers are read:** it counts a page's claims about its own evidence tier, not sites and not agreements. A page may mark one sentence `[O]` and cover twenty addresses with it. It is published as strength 2 because it is the only quantity that is both uniform across all ten pages and mechanically recomputable — and because the alternative was ten rows of silence.

**The counting rule, so it is reproducible:** everything up to and including a page's first line consisting of exactly `---` is the provenance banner and mark legend and is skipped; every occurrence of `[R]`/`[O]`/`[I]` after it counts.

## 3. The tuple table

| subsystem | page | 1 read — `sites ⊇ read ⊇ ported` | 2 agreement | 3 exercised | 4 byte-owned |
|---|---|---|---|---|---|
| **obj writer**<br>`coff` | [`P_COFF.md`](whitebox/ref/P_COFF.md) | **120 sites** (Ghidra function entries in the band)<br>⊇ **read 21** (entries)<br>⊇ **ported RESIDUE**<br>**second denominator** 129 (TU-level, 1.1×) | `[O] 16` of `57` marks (28.1 %)<br>**RESIDUE — no differential exists for coff** beyond the page's own mark census | 871 / 871 workload TUs whose obj real c2 wrote (100.00 %)<br>*OUTPUT PROXY, NOT A SITE COUNT. Every obj in the workload went through this writer, so the proxy is 100 % by construction and carries no information about WHICH of the 120 functions ran. 393,236 section headers over the 871* | CITED `#3534` |
| **section & symbol model**<br>`section` | [`P_SECTION.md`](whitebox/ref/P_SECTION.md) | **137 sites** (Ghidra function entries in the two bands (102 + 35))<br>⊇ **read 24** (entries)<br>⊇ **ported 1 / 15 live .gl record-dispatcher arms the port has a decoder for (6.67 %)**<br>**second denominator** 327 (TU-level, 2.4×) | `[O] 17` of `53` marks (32.1 %)<br>**RESIDUE — no differential exists for section** beyond the page's own mark census | 14 / 14 distinct section names real c2 emits over the workload (100.00 %)<br>*OUTPUT PROXY, NOT A SITE COUNT — and its denominator is the observed set, so it is 14/14 by construction. The names: .drectve .debug$S .XBLD$W:C1 .XBLD$W:C2 .text .text$yc .text$yd .pdata .xdata$x .rdata .rdata$r .data .bss .CRT$XCU* | CITED `#3534` |
| **register allocator**<br>`regalloc` | [`P_REGALLOC.md`](whitebox/ref/P_REGALLOC.md) | **70 sites** (Ghidra function entries in color.c's span)<br>⊇ **read 33** (entries (18 code + 15 data))<br>⊇ **ported RESIDUE**<br>**second denominator** 230 (TU-level, 3.3×) | `[O] 12` of `56` marks (21.4 %)<br>**RESIDUE — no differential exists for regalloc** beyond the page's own mark census | RESIDUE — per-site exercise is unmeasurable: nothing traces c2.dll's own addresses over the workload. The nearest measured thing is P_REGALLOC's own [O] evidence on 6 frozen grid cells (G1-G4, L3, P1), which is a 6-cell probe grid and not the 878-TU workload | CITED `#3534` |
| **globregs: the candidate SET, its ORDER, and the tie key**<br>`globregs` | [`P_GLOBREGS.md`](whitebox/ref/P_GLOBREGS.md) | **19 sites** (the R4 target plus its 18 callees)<br>⊇ **read 26** (entries (16 code + 10 data))<br>⊇ **ported RESIDUE** | `[O] 21` of `74` marks (28.4 %)<br>**RESIDUE — no differential exists for globregs** beyond the page's own mark census | RESIDUE — per-site exercise unmeasurable (no address trace). P_GLOBREGS's own [O] is 262 formal->register assignments over 62 GRID objs — a probe grid, not the 878-TU workload | CITED `#3534` |
| **DAG build + scheduler**<br>`dag` | [`P_DAG.md`](whitebox/ref/P_DAG.md) | **61 sites** (Ghidra function entries in the two bands (48 + 13))<br>⊇ **read 32** (entries (24 code + 8 data/table))<br>⊇ **ported RESIDUE**<br>**second denominator** 83 (TU-level, 1.4×) | `[O] 7` of `50` marks (14.0 %)<br>**RESIDUE — no differential exists for dag** beyond the page's own mark census | RESIDUE — per-site exercise unmeasurable (no address trace). The scheduler band 0x10be5cce-0x10be663f is a TU with NO ICE SITE, so even its attribution is a hypothesis rather than a fact (SUBSYS.md's own blind-spot box) | CITED `#3534` |
| **inliner**<br>`inline` | [`P_INLINE.md`](whitebox/ref/P_INLINE.md) | **93 sites** (Ghidra function entries in the inliner band)<br>⊇ **read 16** (entries)<br>⊇ **ported RESIDUE**<br>**second denominator** 350 (TU-level, 3.8×) | `[O] 17` of `61` marks (27.9 %)<br>PENDING — the inliner's clause-by-clause differential is being built by lane w-inlmetric (decision 15, boards #3623-#3628), in flight at this render. Cited, not waited on, and its worktree is not read<br>*the inliner's clause-by-clause differential is being built by lane w-inlmetric (decision 15, boards #3623-#3628), in flight at this render. Cited, not waited on, and its worktree is not read* | RESIDUE — per-site exercise unmeasurable (no address trace). P_INLINE's own worked case is one anchor (keygen_xbox.cpp) where the read predicts six inlines and gets one [O] — a single TU, not a workload count | CITED `#3534` |
| **instruction encoder (tuple -> one PPC word, plus .text relocation requests)**<br>`encode` | [`P_ENCODE.md`](whitebox/ref/P_ENCODE.md) | **14 sites** (Ghidra function entries in the encoder band)<br>⊇ **read 79** (distinct encode arms (covering 660 of 660 machine opcodes))<br>⊇ **ported 30 / 79 encode arms the port can produce a word through (37.97 %)** | `[O] 9` of `28` marks (32.1 %)<br>630,548 / 634,457 executable .text words explained by the page's own arm masks (99.38 %)<br>*THE STRICT-MASK PASS IS THE ONE WITH TEETH — the page says so itself: a second pass with every read form masked reads 99.8060 % and MUST NOT be quoted as stronger, because sixteen VMX128 forms are masked at 0x03FFFFFF and a generous mask cannot fail. Denominator is 500 objs, NOT the 878-TU workload. The 3,909 residuals are unmasked forms, not disagreements; 0 unexplained at any of 124,700 relocation sites* | 863 / 871 workload TUs with any .text section (99.08 %)<br>*OUTPUT PROXY, NOT A SITE COUNT — 178,104 .text COMDATs over the 863. Says nothing about which of the 79 arms the workload takes* | CITED `#3534` |
| **EH state synthesis**<br>`eh` | [`P_EH.md`](whitebox/ref/P_EH.md) | **47 sites** (Ghidra function entries in the EH band)<br>⊇ **read 19** (entries)<br>⊇ **ported RESIDUE**<br>**second denominator** 127 (TU-level, 2.7×) | `[O] 14` of `41` marks (34.1 %)<br>**RESIDUE — no differential exists for eh** beyond the page's own mark census | 849 / 871 workload TUs carrying .pdata (97.47 %)<br>*OUTPUT PROXY, NOT A SITE COUNT — 103,128 .pdata records over the 849. Its value is the INDEPENDENT CORROBORATION beside it: this census counts .xdata$x in exactly 67 of 871 TUs, reproducing P_EH's own `67 workload objs, all STLport` from a different instrument* | CITED `#3534` |
| **compiler-label numbering (the $M/$T/$L* counter and its charges)**<br>`label` | [`P_LABEL.md`](whitebox/ref/P_LABEL.md) | **163 sites** (charging sites (31 direct calls of the allocator + 132 of the generic ctor))<br>⊇ **read 163** (sites (the population is CLOSED by construction — the allocator's address is never taken))<br>⊇ **ported RESIDUE** | `[O] 11` of `73` marks (15.1 %)<br>**RESIDUE — no differential exists for label** beyond the page's own mark census | RESIDUE — per-site exercise unmeasurable, and WORSE HERE THAN ELSEWHERE: 42 of the 163 sites sit on LOOP BACK EDGES, so a TU's charge is a data-dependent sum over whatever population the loop walks, not a per-construct constant. A site-hit count would not be a charge count even if we had one (P_LABEL §0; LABEL_SEED_GAP is not a constant either) | CITED `#3534` |
| **symbol records: storage class, section number, WEAK EXTERNALS**<br>`symbol` | [`P_SYMBOL.md`](whitebox/ref/P_SYMBOL.md) | **5 sites** (functions (FUN_10b28a9b and its four callees))<br>⊇ **read 27** (addresses)<br>⊇ **ported RESIDUE**<br>**second denominator** 5 (TU-level, 1.0×) | `[O] 4` of `52` marks (7.7 %)<br>**RESIDUE — no differential exists for symbol** beyond the page's own mark census | 675 / 871 workload TUs needing a weak external (77.50 %)<br>*OUTPUT PROXY, NOT A SITE COUNT, and it is CITED from another instrument's key rather than measured by this one — one fact, one locator (docs/GAPS.md §6). It counts TUs that NEED a weak external, not sites of the record writer that ran* | CITED `#3534` |

## 4. `ported` — two rows measured, eight still residue

Decision 15 asks strength 1 for *"how many the port implements"*. Lane
`w-submetric` shipped it as a **named residue on all ten rows** (`#3617`),
because no port↔image site map existed. Lane `w-encmap` (`#3636`–`#3641`,
decision 16) converted the cheapest one; lane `w-secported`
(`#3661`–`#3666`, decision 17) converted the second.

**`encode` is measured: 27 of 79 arms.** The predicate is
`subsys::recount_encode_ported` and it is **recomputed on every run and
every `cargo test`** from `ENCODE_ARMS.txt` × `c2_core::codegen::mop`'s
public tables — a carried number could rot, and a fabricated one is caught
by `control_a_fabricated_ported_is_caught`. **The denominator is the 79
arms and the choice is published rather than silent** — see the caveat in
§3, which also carries the strict (25) and plan-only (28) readings and the
shape of the 52 unmapped.

**`section` is measured: 1 of 15 live `.gl` record-dispatcher arms.** The
predicate is `subsys::recount_section_ported`, recomputed from
`work/w-secported/GLREC_ARMS.tsv` (decoded from the pinned image) × a scan
of `crates/` outside this crate, and a fabrication is caught by
`control_a_fabricated_section_ported_is_caught`. **Its denominator is
published with four rivals**, and the first thing it corrects is a phrase
this file itself used to print: **there are not 27 arms.** 27 is a count of
TAG VALUES; they index 16 jump slots, one of which is the fatal `C1001`
path serving eight tags. The population is **15 live handlers over 19 live
tags plus one refusal over 8** — `P_SECTION.md` §7.

**THE PROPERTY THAT MAKES A ROW CONVERTIBLE IS NOT "ITS SITES ARE
RULES".** `#3636` named that property and `#3661` tested it on the only
other row it predicted for. What both convertible rows actually share is a
**key the port carries on its own side**: an encode-form number, adopted
from c2's table under `DISCLOSURE.md` W-MOP-2, and a `.gl` arm address,
adopted under W-ALIAS-1. Where no adoption exists there is nothing to join
on, and `P_SECTION.md` §7.4 makes that concrete — the port implements the
alignment-nibble ladder and the `.bss` reversal rule, **derived black-box
and citing nothing**, so on a site unit an address grep scores them 0 where
the honest answer is 1. **The two rules agree with c2 and are joinable to
it by nothing.**

**The other eight stay residue and the reason is structural, not a gap**: the
port is **I/O-behavioral by construction** (`CLAUDE.md`'s one correctness
rule — AVX, restructured CFGs, anything, so long as the *output obj*
matches), so *"the port implements site `0x10b2e7f8`"* has no truth value
for most of these addresses. **A row where the question is not well formed
keeps its residue rather than getting an invented number**, and `verify`
refuses a `ported` number that nothing can recount.

Per residue row, with the reason rather than a blank:

* **`coff`** — no port<->image site map for the obj writer. crates/c2-obj writes COFF by a route derived from the format, not from these 21 addresses; counting which of them the port implements needs the derived-vs-fitted provenance census, which lane w-provenance owns this wave
* **`regalloc`** — the port has no register allocator of this shape at all — the byte-exact classes are one-function bodies whose registers are assigned by codegen::select_function's own rules, not by a colouring pass. A site-level numerator is not merely unmeasured, it is not yet defined
* **`globregs`** — the port does no global register promotion; there is no site to count. P_GLOBREGS §2's order and tie key are read but unadopted
* **`dag`** — the port schedules nothing — emission order is tuple-list order (P_BLOCKORDER §5.2, #3437-#3441) and the port's bodies are built straight-line. No site-level numerator is defined
* **`inline`** — the port carries a FITTED inline predicate (INLINE_PREDICATE.md's 0.9716 model), not an implementation of these 93 sites. The clause-by-clause port-state column is lane w-inlmetric's deliverable this wave and is not built here
* **`eh`** — P_EH marks two entries `[O] port` — the port reproduces the deferred unwind-word pass's OUTPUT — but the page's marks are per-claim, not per-site, so they do not compose into a `sites implemented` numerator. Building one is the same missing port<->image map as every other row
* **`label`** — the port mints labels from its own counter; no mapping exists from its mint points to these 163 charging sites. LABEL_COUNTER.md's own finding is that stride == minted fails both ways, so a naive site count would be wrong even if it were built
* **`symbol`** — P_SYMBOL §2 marks several addresses `[O]` via the port's own ObjImage::weak_externals with KNOWN-ANSWER 0 alarms, so parts of this subsystem ARE implemented and graded — but per-ADDRESS, and the page's 27 addresses do not map onto port functions one-for-one. The numerator is undefined rather than zero

And for each **measured** row, the caveat that carries its denominator choice and the rivals it was chosen against — verbatim, because a denominator published only in the source of the tool that prints it is not published:

* **`section` — 1 of 15 live .gl record-dispatcher arms the port has a decoder for**
  * source: lane w-secported, board #3661-#3666: work/w-secported/GLREC_ARMS.tsv (decoded from the pinned image on this tree) x a scan of crates/ outside c2-harness
  * THE DENOMINATOR IS THE 15 LIVE ARMS OF THE .gl RECORD DISPATCHER 0x10b9b8e9, AND `27 arms` -- which this row itself used to say and decision 17 repeats -- IS NOT AN ARM COUNT. Re-measured from the image: 27 TAG VALUES (0x01..0x1B) index a 27-entry byte table into 16 jump slots, and ONE slot is the fatal C1001 path 0x10b9c5ca serving EIGHT tags (0x0C 0x0F 0x11 0x13 0x14 0x15 0x16 0x17). So the population is 15 live arms over 19 live tags plus one refusal over 8, and calling it 27 overstates the arm count by 1.8x. FOUR RIVAL DENOMINATORS WERE MEASURED AND ARE PUBLISHED RATHER THAN ONE CHOSEN SILENTLY: 16 arms counting the refusal as a site (a port that also refuses agrees with c2 by doing nothing, so this is rejected); 27 tags, on which the port names 5 (0x01 0x02 0x04 0x0E 0x10, all in c2_il::func::glalias, and three of them as pattern LOCATORS rather than decoders); the page's 24/25 read ENTRIES, on which an address grep gives 2 and is WRONG -- see recount_section_ported for the two rules the port implements while citing nothing; and the band's 137 / the TU-level 327, neither of which the port maps onto at all. CONTAINMENT: the 15 arms all live INSIDE 0x10b9b8e9, which is one of the 24 read entries, which is one of the 137 sites -- so `sites superset-of read superset-of ported` holds as a containment of SITE SETS, while the three counts are in three granularities and their RATIOS must not be compared. THE ONE ARM IS 0x10b9bdcf, the shared tag-0x04/0x0E/0x10 handler, decoded by c2_il::func::glalias under DISCLOSURE W-ALIAS-1. The 14 unported arms include 0x10b9c212 -- TAG 0x09, THE SECTION DEFINITION RECORD, every field of which P_SECTION marks obj-checked by .gl mutation. The port does not read it: it carries 17 fully-resolved (name, Characteristics) constants where c2 has a kind switch, a remapper, a base resolver and an alignment chooser. See P_SECTION.md section 7

* **`encode` — 30 of 79 encode arms the port can produce a word through**
  * source: lane w-encmap, board #3636-#3641 (27/79); lane w-encarms, board #3756-#3761 (29/79, wave 18); lane w-fmadd, board #3790-#3795 (30/79, wave 19): ENCODE_ARMS.txt (79 rows, re-measured on this tree) x c2_core::codegen::mop::{plan, OPCODES}
  * THE DENOMINATOR IS THE 79 ARMS, NOT THE BAND'S 14 AND NOT THE 111 JUMP-TABLE ENTRIES, and the choice is published rather than silent: `read` on this row is already 79 arms, so `read superset-of ported` is well formed only in the arm unit. The 111 entries are 111 FORMS, each belonging to exactly one arm (re-measured: 111 -> 79, no form served by two arms), so the entry unit would count the same arm up to 12 times. The band's 14 is Ghidra function entries -- a different population entirely. AN ARM COUNTS ON ONE OF ITS FORMS, so this OVER-states partial arms: the strict reading (every form of the arm reachable) is 27, and the loose reading (a FieldPlan exists, whether or not an opcode reaches it) is 31 -- the extra arm is 10bfa26c, form 2, a plan no opcode reaches. BOTH MOVED BY ONE ON 2026-08-29 and both were re-derived, not incremented: arm 10bfa49a serves form 24 ALONE, so an arm that counts published also counts strict, which is why w-fmadd's adoption is the first that moves all three readings together. 27 -> 29 ON 2026-08-28, lane w-encarms: arms 10bfa285 (form 7, `bl`) and 10bfa76a (form 54, `mfspr`) were read at their addresses in the pinned image and adopted, discharging all three of codegen::word_seam's armed refusals; DISCLOSURE W-ENCARMS-1. THE SAME LANE REFUTED THE `25` THIS CAVEAT USED TO CARRY FOR THE STRICT READING: measured directly at master 4b79bf46a it is 24, so the strict reading moved 24 -> 26 and the published 25 never reproduced (board #3759). The 50 still-unmapped arms are NOT uniform, and neither is what the WORKLOAD reaches: over 861 real-c2 objs of the 878-TU workload (3,192,747 non-zero executable .text words) only 10 of the 52 pre-adoption unmapped arms are reached unambiguously and 31 are reached ZERO times, including the ICE arm 10bfa81d and 30 others; the 104-opcode default arm 10bf9f91 is reached by at most 10 words, 0.0003 %. work/w-encarms/armhist.py. 29 -> 30 ON 2026-08-29, lane w-fmadd: arm 10bfa49a (form 24, the FUSED multiply-adds) was read at its address and adopted, and unlike w-encarms's two this one had NO field plan and NO opcode row before the lane -- the port could not compose the word at all. It is the fourth-most-reached unmapped arm on the same 861-obj histogram (7,995 words, unambiguous), and the adoption is an EMIT and not a fold: `a*b+c` was a refusal and is now byte-exact (fixtures/cpp/w13c_fma.cpp, 17 functions). DISCLOSURE W-FMADD-1. See P_ENCODE.md section 10


## 5. Where `SUBSYS.md` §1's own cell needs reading twice

Found by re-measuring every denominator on this tree rather than carrying it.
**None of these is corrected here, and that is a rule rather than a fence** —
a disagreement recorded beside a page beats a silent rewrite of it
(`#3538`). It held even where a lane DID own the page: `w-secported` owns
`P_SECTION.md`, found its coverage line unreproducible under all three
readings of its own table and wrong from the file's first commit, and left
the line as written with the amendment beside it at §7.5.

* **`section`** — THE PAGE'S OWN COVERAGE LINE DOES NOT REPRODUCE, AND NEVER DID. `read 24` is carried verbatim from P_SECTION.md:11 (`24 entries against a denominator of 137`) and recounting the page's §1 table on this tree gives 25 ROWS, of which 22 are Ghidra function entries (three -- 0x10b9bdcf, 0x10b9c212, 0x10b9c5ca -- are addresses INSIDE 0x10b9b8e9 and the page says so), of which 20 are inside the two bands that give the 137 (0x10b805b3 is misc.c, 0x10c27b56 is smdmisc.c). 24 reproduces under NONE of the three, and `git log -S` puts the line at 25 rows in the file's FIRST commit -- wrong when written, not rotted since, the same family as #3643. NOT CORRECTED HERE: the line is this row's den_probe and the standing convention is that an amendment stands beside the original reading (P_SECTION §5's own retraction is the model); it is flagged on the page at §7.5 instead. `ported` is in a THIRD unit again -- 15 live dispatcher arms -- and the containment holds as a nesting of SITE SETS, not as comparable ratios

* **`regalloc`** — SUBSYS.md §1 prints `33 / 70`; the page's 33 is 18 code + 15 data entries, and the 15 data entries are TABLES, not functions, so the numerator and the denominator are in different units. Read as entries-against-functions, not as a fraction. Also: the band reproduces 70 only HALF-OPEN (71 inclusive) — 0x10b3219f is dag.c's anchor

* **`globregs`** — THE READ IS LARGER THAN ITS OWN DENOMINATOR (26 against 19) and the page says why in its own words: the read went OUTSIDE the registered denominator on purpose, because the three functions that decide the order are not callees of the target at all. The page's honest statement is `6 of 18 callees read to policy level, plus 7 functions outside the target's subtree`. SUBSYS.md §1's cell `16 code + 10 data` prints no denominator at all

* **`encode`** — SUBSYS.md §1 prints `14 / 14`, which is the BAND (14 Ghidra entries, recounted here and correct). The page's own coverage line is `79 of the 79 distinct arms`, covering `660 of 660` opcodes. THE TWO CELLS ARE IN DIFFERENT UNITS and neither is wrong; a reader taking `14 / 14` for the coverage statement is off by a factor of 5.6 in the numerator and 47 in the opcode denominator

* **`label`** — SUBSYS.md §1's cell reads `163 sites / 86+25 callers`. The 86 reproduces on the page (`All 132 are direct E8 calls from 86 distinct functions`, P_LABEL:445/471). THE `25` DOES NOT REPRODUCE ANYWHERE ON THE PAGE — the nearest figure is 85, the PLACEMENT population that calls FUN_10bd415e (P_LABEL:505), and the nearest literal 25 on the page is `fitted from 25 TUs` in an unrelated sentence at :222. Reported, not corrected: P_LABEL/SUBSYS.md are not this lane's to edit

* **`symbol`** — SUBSYS.md §1 prints `27 / 5`, a ratio greater than 1: the numerator is ADDRESSES and the denominator is FUNCTIONS. Recounted here, the page's own address band 0x10b28a9b-0x10b28d6f holds exactly ONE Ghidra function entry, so there is no band reading under which `5` is a function count of that span — the 5 is FUN_10b28a9b plus four callees that live elsewhere in coff.c's gap


## 6. Workload stamp

| what | value |
|---|---|
| whitebox ref index | `docs/whitebox/ref` |
| workload section census | `871` records, generated `2026-08-04T10:06:18Z` |
| corpus | `940d07dcb0960964ad61aa5f025658f993eb46b2` dirty=`false` |
| recomputed here | 871 TUs, 393,236 sections, 14 distinct names, `nsec-disagree 0` (known answer 0) |
| byte-owned | **CITED, NOT RE-MEASURED** — #3534 (w-permeasure, 2026-08-25, port tree a8593651b, 878-TU workload): the port's wrong bodies are 1,968 bodies / 7,912 substituted words, opcode 7,902 = 99.87 %, 0 pure reorderings, 92.78 % wrong at word 0. docs/DIFF_STRUCTURE.md, docs/PERMUTER_POPULATION.md §3 |

## 7. Machine-readable keys

Namespaced, and **sorted by key NAME rather than by mass** — this repo's
standing rule against dispatching off a blocked-key size ranking, which has
now bound five times (`#3505`, and *"ranking instruments measure
themselves"*, four for four).

```text
subsys-metric byte-owned CITED-3534
subsys-metric coff-exercised-proxy 871
subsys-metric coff-exercised-proxy-den 871
subsys-metric coff-marks-obj 16
subsys-metric coff-marks-total 57
subsys-metric coff-ported RESIDUE
subsys-metric coff-read 21
subsys-metric coff-sites 120
subsys-metric coff-sites-recounted 120
subsys-metric coff-sites-tu-level 129
subsys-metric dag-exercised-proxy RESIDUE
subsys-metric dag-marks-obj 7
subsys-metric dag-marks-total 50
subsys-metric dag-ported RESIDUE
subsys-metric dag-read 32
subsys-metric dag-sites 61
subsys-metric dag-sites-recounted 61
subsys-metric dag-sites-tu-level 83
subsys-metric eh-exercised-proxy 849
subsys-metric eh-exercised-proxy-den 871
subsys-metric eh-marks-obj 14
subsys-metric eh-marks-total 41
subsys-metric eh-ported RESIDUE
subsys-metric eh-read 19
subsys-metric eh-sites 47
subsys-metric eh-sites-recounted 47
subsys-metric eh-sites-tu-level 127
subsys-metric encode-exercised-proxy 863
subsys-metric encode-exercised-proxy-den 871
subsys-metric encode-marks-obj 9
subsys-metric encode-marks-total 28
subsys-metric encode-ported 30
subsys-metric encode-ported-den 79
subsys-metric encode-read 79
subsys-metric encode-sites 14
subsys-metric encode-sites-recounted 14
subsys-metric globregs-exercised-proxy RESIDUE
subsys-metric globregs-marks-obj 21
subsys-metric globregs-marks-total 74
subsys-metric globregs-ported RESIDUE
subsys-metric globregs-read 26
subsys-metric globregs-sites 19
subsys-metric inline-exercised-proxy RESIDUE
subsys-metric inline-marks-obj 17
subsys-metric inline-marks-total 61
subsys-metric inline-ported RESIDUE
subsys-metric inline-read 16
subsys-metric inline-sites 93
subsys-metric inline-sites-recounted 93
subsys-metric inline-sites-tu-level 350
subsys-metric label-exercised-proxy RESIDUE
subsys-metric label-marks-obj 11
subsys-metric label-marks-total 73
subsys-metric label-ported RESIDUE
subsys-metric label-read 163
subsys-metric label-sites 163
subsys-metric regalloc-exercised-proxy RESIDUE
subsys-metric regalloc-marks-obj 12
subsys-metric regalloc-marks-total 56
subsys-metric regalloc-ported RESIDUE
subsys-metric regalloc-read 33
subsys-metric regalloc-sites 70
subsys-metric regalloc-sites-recounted 70
subsys-metric regalloc-sites-tu-level 230
subsys-metric section-exercised-proxy 14
subsys-metric section-exercised-proxy-den 14
subsys-metric section-marks-obj 17
subsys-metric section-marks-total 53
subsys-metric section-ported 1
subsys-metric section-ported-den 15
subsys-metric section-read 24
subsys-metric section-sites 137
subsys-metric section-sites-recounted 137
subsys-metric section-sites-tu-level 327
subsys-metric subsystems 10
subsys-metric symbol-exercised-proxy 675
subsys-metric symbol-exercised-proxy-den 871
subsys-metric symbol-marks-obj 4
subsys-metric symbol-marks-total 52
subsys-metric symbol-ported RESIDUE
subsys-metric symbol-read 27
subsys-metric symbol-sites 5
subsys-metric symbol-sites-tu-level 5
subsys-metric verify-failures 0
subsys-metric workload-census-nsec-disagree 0
```

## 8. Self-verification

`VERIFY: PASS` — 7 band denominators recounted from `FUNCS.tsv`, 10 pages' coverage probes found verbatim, 10 mark censuses, 0 empty residues.

### 8.1 The controls, and that they were watched failing

`#3336`: **a control never seen failing is decoration.** Seven fabrications
run on every `cargo test -p c2-harness --lib subsys`, each asserting the
verifier *refuses*, and each pinned to the check that must own the refusal so
a case cannot pass by being caught for the wrong reason:

| control | fabrication | must be caught by |
|---|---|---|
| `control_a_fabricated_denominator_is_caught` | the inliner's `93` → `94` | the `FUNCS.tsv` recount |
| `control_a_dropped_subsystem_is_caught` | the `eh` row deleted from the table | the `SUBSYS.md` §1 enumeration |
| `control_an_empty_residue_is_caught` | `dag`'s `ported` residue set to `"   "` | the no-silence check |
| `control_a_moved_coverage_line_is_caught` | `P_COFF`'s probe pointed at a line that is not on the page | the verbatim probe |
| `control_a_fabricated_ported_is_caught` | `encode`'s `ported` `27` → `28` | the `ENCODE_ARMS.txt` × `mop` recount |
| `control_a_ported_number_with_no_recount_is_caught` | a number typed into `coff`'s `ported`, which has no recount | the recount-or-residue rule |
| `control_a_fabricated_section_ported_is_caught` | `section`'s `ported` `1` → `2` | the `GLREC_ARMS.tsv` × `crates/` scan |

**Every `ported` control was watched failing against the SHIPPED table,
not only against a copy** (lanes `w-encmap`, `w-secported`): editing the real
`encode` cell `27` → `28` reddens four tests with
`encode: ported DOES NOT REPRODUCE — table says 28/79, the tree gives 27/79`,
and deleting the `OPCODES` half of `port_reaches_form` reddens five,
including `the_three_ported_readings_are_distinct` (`published` collapses
onto `planned` at 28). Editing the real `section` cell `1` → `2` likewise
reddens four with `section: ported DOES NOT REPRODUCE — table says 2/15, the
tree gives 1/15`. A recount that only ever grades a mutated copy is a
recount that has never been shown to bind the number the doc prints.

**Two further checks are not fabrications and are worth naming separately.**
`the_section_ported_arm_is_the_one_the_port_actually_decodes` pins *which*
arm the numerator found — a count that is right for the wrong reason is
`#3336`'s other failure — and pins the dispatcher's shape (27 tags, 16
slots, 15 live arms, 19 live tags, 8 fatal) so a re-dump that disagrees
reddens instead of shipping a moved denominator quietly.
`the_observer_crate_cannot_move_its_own_ported` is **`#3641`'s family, caught
by construction rather than by review**: the `section` numerator is a scan of
source text for arm addresses, and `subsys.rs` must name those addresses to
explain itself — so the scan excludes its own crate. **The hazard was
measured, not assumed**: disabling that one exclusion moves the shipped
number from `1/15` to `2/15`.

And `scripts/subsys_metrics.sh --self-test` drives the **binary** against
three deliberately corrupted copies of the reference index — a function moved
out of the inliner band, `P_EH.md`'s coverage line edited, a subsystem
deleted from `SUBSYS.md` §1 — requiring each to exit non-zero *and* proving
each mutation applied first, because a `sed` that matched nothing leaves a
clean copy and the case then "passes" by testing the control twice
(`#3516`'s mutation-not-applied failure, named in the same words by
`scripts/gate_identity_diff.sh --self-test`).

**And one control is not a fabrication at all**, because the defect it
guards is not a number.
`control_a_measured_ported_must_reach_the_rendered_table` asserts that §3's
own row carries each measured `ported`. That check did **not** exist while
§3's cell was the hard-coded string `ported RESIDUE` on all ten rows — so
for two waves this file printed `RESIDUE` in the table and *"encode is
measured: 27 of 79 arms"* four paragraphs below it, with the
machine-readable keys siding with the prose. **Every other control here
fabricates a NUMBER; that defect fabricated a WORD**, which is the blind
spot `#3641` and `#3643` also sit in. `#3665`, repaired and watched failing
by restoring the literal.

## 9. How to regenerate

```sh
scripts/subsys_metrics.sh              # console report
scripts/subsys_metrics.sh --write      # regenerate THIS FILE
scripts/subsys_metrics.sh --keys       # only the subsys-metric lines
scripts/subsys_metrics.sh --self-test  # prove the verifier CAN go red

cargo test -p c2-harness --lib subsys  # the same checks, plus the 4 controls
cargo run -p c2-harness --bin c2rs -- subsys
```

**No toolchain, no capture, no scan.** The instrument reads
`docs/whitebox/ref/` and the committed workload section census and prints, so
it degrades cleanly by construction: an absent census makes every output
proxy read `NO-DATA`, never `0`.

### 9.1 `#1406`, and why this is not in `gate.sh`

`#1406` binds any instrument whose output is quoted as evidence to run under
`cargo test` or `scripts/gate.sh`. §0 forbids the second. The resolution is
`decode-reach`'s, and it is the reason this file's numbers are trustworthy
without the gate grading them: **the logic and the controls live in
`crates/c2-harness/src/subsys.rs` and run under `cargo test --workspace`,
which is a `gate.sh` row.** The verdict they contribute to is `cargo test`'s
— that every denominator here still reproduces from the tree — never the
differential's. `scripts/subsys_metrics.sh` is a thin wrapper over the same
code, so there is **one producer** of the table and it cannot drift from the
tests that grade it.

