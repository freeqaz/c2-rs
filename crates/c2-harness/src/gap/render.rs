//! The scan's own printed report of the factor model. Split out of `gap.rs`
//! unchanged; see [`super`] for the module docs.

use super::fnbytes::byte_fraction_exact;
use super::factors::CfgReach;
use super::{GapReport, TuClass, TuResult, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

/// **THE OBJECT-PLAN BLOCK** (lane `w-objplan`) — the manifest curve, and the
/// named control that keeps it honest.
///
/// # What is printed and why each line is there
///
/// * The **named control first**, always, because a set difference against
///   `docs/plan/CONTROL_TUS.txt` means the tree or the workload stamp moved
///   under the reader and every number below it is about a different corpus.
///   Printed as a **count with the names**, never as a status.
/// * Per component: `observable ⊇ known ⊇ exact`, `differs` derived, and
///   `distinct`. Three denominators and never a bare ratio — a percentage whose
///   denominator is not beside it is board #213's `+82` shape.
/// * A component at `distinct == 1` is labelled **FREE** and must not be read
///   as progress: it takes one value across the whole workload, so agreeing
///   with it is agreeing with a constant.
/// * The observe-side inventory, which is not a curve at all: it is the honest
///   description of the population the un-conjuncted lanes will have to serve,
///   and it re-derives figures (`weak externals`, `COMDAT`) this project has
///   only ever carried.
///
/// **`plan-*` is NECESSARY but NOT SUFFICIENT for `match`** and the block says
/// so on every run. It is an instrument; the byte judge is unchanged.
/// **DECODE REACH — its own block, under its own disclaimer** (lane
/// `w-decodereach`, decision 13's row 4a(i) / I1).
///
/// `docs/FUNCTION_BYTE_MATCH.md` §0 is the standing template for every gradient
/// added after FBM and this is one: never in `scripts/gate.sh`, namespaced
/// keys, **licenses no emit**, and `NO-RESULT` — never a zero — over an empty
/// scan.
fn render_decode_reach(report: &GapReport) {
    let d = |k: &str| report.decode_total(k);
    let (observable, reached, stopped, nobody) = (
        d("decode-reach-observable"),
        d("decode-reach-reached"),
        d("decode-reach-stopped"),
        d("decode-reach-nobody"),
    );
    let graded = reached + stopped;
    // **THE POSITIVE CHECK, FIRST.** "The run must have GRADED something" —
    // never an enumeration of the ways it can be empty. A zero here prints
    // NO-RESULT loudly and publishes no number at all.
    if graded == 0 {
        println!(
            "\n\x20 DECODE REACH: NO-RESULT — the general decode was offered NO body with a \
             body to decode ({observable} rows observed, {nobody} of them bodiless).\n\
             \x20   Nothing was graded. This is not `decode-reach-stopped 0`; a lane quoting a \
             reach number off this scan has no number."
        );
        return;
    }
    println!(
        "\n\x20 DECODE REACH (w-decodereach) — the progress signal for the GENERAL DECODE \
         (decision 13, row 4a(i) / I1). A CHARACTERIZATION instrument, NEVER a gate, and it \
         LICENSES NO EMIT.\n\
         \x20   REACH IS NOT ADMISSION and must never become it. A body that decodes is not a \
         body the port may emit — a wrong emit scores strictly below the refusal it replaced \
         (`docs/PROGRESS_METRIC.md`). The byte judge is unchanged: real `c2.dll` under wibo.\n\
         \x20   Read the THREE DENOMINATORS as a containment, never as a ratio: \
         observable ⊇ reached ⊇ verified."
    );
    let pct = |n: usize, den: usize| {
        if den == 0 {
            "n/a".to_string()
        } else {
            format!("{:.2}%", 100.0 * n as f64 / den as f64)
        }
    };
    let (badm, brea, bmod) = (
        d("decode-reach-bytes-observable"),
        d("decode-reach-bytes-reached"),
        d("decode-reach-bytes-modeled"),
    );
    let modeled = d("decode-reach-modeled");
    println!(
        "\x20   DECODER = `{}` — every number below is a property of THIS decoder. When the \
         seam is replaced by a stronger one the reach is expected to DROP before it climbs; a \
         drop across that boundary is a change of instrument, not a regression.\n\
         \x20   ALL BODIES     observable {observable}  FRAME-reached {reached} ({})  \
         MODEL-reached {modeled} ({})  stopped {stopped}  nobody {nobody}\n\
         \x20   …BY BYTE       observable {badm}  frame {brea} ({})  model {bmod} ({}) — a body \
         count and a byte count are two denominators and neither is quoted alone\n\
         \x20     FRAME reach = the walk landed on the segment tail. MODEL reach = …AND every \
         operand was in the decoder's modeled vocabulary. **Quote them together.** Frame reach \
         is a framing claim and is nearly saturated here; model reach is the one with headroom, \
         and it is the reading row 4a(i) is funded to move.",
        super::decode::DECODER,
        pct(reached, observable),
        pct(modeled, observable),
        pct(brea, badm),
        pct(bmod, badm),
    );
    let (eobs, erea, ever) = (
        d("decode-reach-emit-observable"),
        d("decode-reach-emit-reached"),
        d("decode-reach-verified"),
    );
    let (emod, evermod) = (
        d("decode-reach-emit-modeled"),
        d("decode-reach-verified-modeled"),
    );
    println!(
        "\x20   EMITTED ONLY   observable {eobs}  frame {erea} ({})  model {emod} ({})  VERIFIED \
         {ever} ({} of frame-reached) — `verified` is the BYTE JUDGE's own word \
         (`FnByte::Exact`: bytes AND relocations), asked of the bodies the decode reached. \
         unbound {} (published, never folded — {eobs} + unbound is the whole emitted census)\n\
         \x20     …of the {ever} VERIFIED, {evermod} are also MODEL-reached. A body the judge \
         calls exact that the decode does not model is a body whose bytes were NOT bought by \
         modelling it.",
        pct(erea, eobs),
        pct(emod, eobs),
        pct(ever, erea),
        d("decode-reach-emit-unbound"),
    );
    // **THE THIRD STRENGTH and the I1 DIVERGENCE DETECTOR** (`w-unfuse`'s
    // `decode_bodies` seam, board #3555).
    let (gram, gna, ang) = (
        d("decode-reach-grammar"),
        d("decode-reach-grammar-not-admitted"),
        d("decode-reach-admitted-not-grammar"),
    );
    println!(
        "\x20   GRAMMAR-reached {gram} ({}) — off `IlBundle::decode_bodies`, a SECOND parse \
         over the census's own segmentation.\n\
         \x20     ADMISSION THAT IS NOT DECODE: grammar-not-admitted {gna}  \
         admitted-not-grammar {ang}. **{gna} IS THE BASELINE, NOT ZERO**, and it is the size \
         of a THIRD layer `w-unfuse`'s split does not reach: the grammar reads these bodies \
         WHOLE and the census refuses them at `shape_to_function`'s SYMBOL BINDING, downstream \
         of `AdmissionPolicy`. Every one is an `:eof` key. `reached_shape() == is_admitted()` \
         IS zero by construction — but that is a tautology (`is_admitted` is defined as \
         `reached_shape`), and a criterion that cannot fail has abstained, not passed \
         (`#3336`). This pairing is against the CENSUS's verdict and it can fail; it does.\n\
         \x20     So the I1 signal is the CHANGE in {gna}, measured against this baseline — \
         never against 0.\n\
         \x20     THE THREE STRENGTHS ARE NOT A CHAIN: grammar∧model {}  grammar-not-model {} \
          model-not-grammar {}. `frame ⊇ model` holds; `grammar` contains neither and is \
         contained in neither. Three questions, three numbers, no ladder.",
        pct(gram, observable),
        d("decode-reach-grammar-and-model"),
        d("decode-reach-grammar-not-model"),
        d("decode-reach-model-not-grammar"),
    );
    let gna_rows = report.decode_rows_by_name("decode-reach-grammar-not-admitted|");
    if !gna_rows.is_empty() {
        println!(
            "\x20     …and WHICH census verdict refuses a body whose grammar the decode \
             reached — {} distinct, sorted by name. A count of disagreements that cannot be \
             looked at is not a repair set:",
            gna_rows.len()
        );
        for (k, n) in gna_rows.iter().take(10) {
            println!("\x20       {n:>8}  {k}");
        }
    }
    // `ROADMAP_SLICING` §3's own predicate, measured here so the two can be
    // compared instead of guessed at.
    let (inm, offm, inden) = (
        d("decode-reach-inmodel"),
        d("decode-reach-offmodel"),
        d("decode-reach-inmodel-denominator"),
    );
    println!(
        "\x20   §3's PREDICATE  in-semantic-model {inm} ({} of {inden} scanned bodies)  \
         off-model {offm} ({}) — this is `ROADMAP_SLICING_2026-08-21.md` §3's *\"≥1 operand \
         outside the semantic model\"*, negated, over ITS population (every body a walk ran \
         on, finished or not). **It is WIDER than MODEL reach above**, which also requires \
         the walk to have reached the tail. Two predicates, two denominators, published side \
         by side so neither can be taken for the other.",
        pct(inm, inden),
        pct(offm, inden),
    );
    // **THE CONTAINMENT NOBODY HAS ASKED**, and its own denominator beside it.
    let (adm, admr, admn) = (
        d("decode-reach-admitted"),
        d("decode-reach-admitted-reached"),
        d("decode-reach-admitted-not-reached"),
    );
    println!(
        "\x20   ADMITTED ⊆ REACHED?  admitted {adm}  of which reached {admr}  NOT REACHED \
         {admn} ({})\n\
         \x20     This is NOT a known-answer-0 control. A body the incumbent parser accepts \
         WHOLE that the general decode stops inside is two independent walkers disagreeing \
         about the same bytes — a finding about the DECODE, not an alarm about this \
         instrument.",
        pct(admn, adm),
    );
    // **THE DISCRIMINATING CELL.** Printed with the threshold that reads it.
    let sep = d(super::decode::SEPARATION_KEY);
    println!(
        "\x20   SEPARATION     reached-and-NOT-admitted {sep}  vs admitted {adm} — the \
         discriminating cells. If this were 0 the instrument would be measuring ADMISSION \
         wearing a reach key's name, which is `#3336` at program scale and the exact failure \
         4a's risk column names.{}",
        if sep == 0 {
            "  ** ZERO — THIS INSTRUMENT IS NOT MEASURING REACH **"
        } else {
            ""
        }
    );
    println!(
        "\x20   controls (all known answer 0): partition-broken {}  population-broken {}  \
         containment-broken {}   ·   GRADED {graded} cells",
        d("decode-reach-partition-broken"),
        d("decode-reach-population-broken"),
        d("decode-reach-containment-broken"),
    );
    // The SECOND DERIVATION (#3288): the same quantity off a walk this lane did
    // not write.
    let incumbent = report.cflow_decoded_totals().0;
    println!(
        "\x20   second derivation: `GapReport::cflow_decoded_totals` (prose since long before \
         this module, off a different map, by code this lane did not write) reads {incumbent}; \
         this walk reads {reached}; disagreement {}. Known answer 0 — computing one quantity \
         two ways and diffing them has caught a wrong figure in every lane that ran it (#3288).",
        incumbent.abs_diff(reached),
    );
    // The first-blocker histogram — labelled, and sorted by NAME.
    let stops = report.decode_rows_by_name("decode-reach-stop|");
    println!(
        "\x20   where the decode STOPPED — {} distinct productions over {stopped} bodies. \
         **A FIRST-BLOCKER KEY IS NOT A DISTANCE AND NOT A RANKING** (#3131: the port stops \
         at its first refusal by design, so every stopped body names exactly one production \
         however many it has; 19 greedy rungs off such a histogram bought reach ZERO). Sorted \
         by NAME, never by mass (#3505, bound five times). The byte row above is the distance:",
        stops.len()
    );
    if stops.is_empty() {
        println!("\x20     (none)");
    }
    for (k, n) in stops.iter().take(16) {
        println!("\x20     {n:>8}  {k}");
    }
    if stops.len() > 16 {
        println!(
            "\x20     … and {} more, all in `gap-metric`-adjacent per-TU rows",
            stops.len() - 16
        );
    }
    // The admitted-not-reached decomposition: a repair set, not a count.
    let anr = report.decode_rows_by_name("decode-reach-admitted-not-reached|");
    if !anr.is_empty() {
        println!(
            "\x20   …and WHICH production stopped an ADMITTED body — {} distinct, sorted by \
             name. A count of disagreements that cannot be looked at is not a repair set:",
            anr.len()
        );
        for (k, n) in anr.iter().take(12) {
            println!("\x20     {n:>8}  {k}");
        }
    }
    // The 2xN judge cross — the emit-path consumer, printed whole.
    let cross = report.decode_rows_by_name("decode-reach-emit|");
    println!(
        "\x20   IS WHAT IT REACHES RIGHT? — the byte judge's own verdict crossed with reach, \
         over c2's own emitted COMDAT leaders. The `refused` column is the denominator on \
         which the judge CANNOT speak and is printed as a number rather than left out:"
    );
    if cross.is_empty() {
        println!("\x20     (none — no emitted function was bound)");
    }
    for (k, n) in cross.iter() {
        println!("\x20     {n:>8}  {k}");
    }
}

/// **SYMBOL BINDING — its own block, under its own disclaimer** (lane
/// `w-symbind`, decision 14).
///
/// `docs/FUNCTION_BYTE_MATCH.md` §0 is the standing template for every gradient
/// added after FBM and this is the fifth: never in `scripts/gate.sh`,
/// namespaced keys, **licenses no emit**, and `NO-RESULT` — never a zero — over
/// an empty scan.
fn render_symbind(report: &GapReport) {
    let s = |k: &str| report.symbind_total(k);
    let observable = s("symbind-observable");
    // **THE POSITIVE CHECK, FIRST.** "The run must have GRADED something" —
    // never an enumeration of the ways it can be empty. A zero here prints
    // NO-RESULT loudly and publishes no number at all.
    if observable == 0 {
        println!(
            "\n\x20 SYMBOL BINDING: NO-RESULT — no census row was paired against a relaxed one \
             ({} TUs walked, {} desyncs). Nothing was graded. This is not `symbind-fused 0`; a \
             lane quoting a symbol-binding number off this scan has no number.",
            s("symbind-tus-scanned"),
            s("symbind-census-desync"),
        );
        return;
    }
    let pct = |n: usize, den: usize| {
        if den == 0 {
            "n/a".to_string()
        } else {
            format!("{:.2}%", 100.0 * n as f64 / den as f64)
        }
    };
    let levels: Vec<(String, usize)> = report.symbind_rows_by_name("symbind-relax-level|");
    let level = levels
        .iter()
        .map(|(k, n)| format!("{k} ({n} TUs)"))
        .collect::<Vec<_>>()
        .join(" + ");
    let (in_class, fused, residue, mono) = (
        s("symbind-in-class"),
        s(super::symbind::FUSED_KEY),
        s("symbind-residue"),
        s("symbind-monotonicity-broken"),
    );
    println!(
        "\n\x20 SYMBOL BINDING (w-symbind) — the THIRD layer, measured (decision 14). A \
         CHARACTERIZATION instrument, NEVER a gate, and it LICENSES NO EMIT: decision 14 says \
         in its own words that this lane *\"measures a refusal population and may not convert \
         it\"*. A wrong emit scores strictly below the refusal it replaced \
         (`docs/PROGRESS_METRIC.md`).\n\
         \x20   THE PAIRING: the STRICT census's verdict x the RELAXED one \
         (`c2_il::Relax`), row by row over one segmentation. The relaxation's ENTIRE content is \
         *supply a placeholder NAME where `.gl` had none*; no grammar widens, no operand \
         vocabulary widens, no byte of the body is re-read. So `fused` means **a name is the \
         only thing between this body and the incumbent admission predicate**.\n\
         \x20   RELAXATION LEVEL = {level} — every number below is a property of THIS level. At \
         level 0 the relaxed census IS the strict one, so `fused` must read 0; that arm is the \
         instrument's own identity control."
    );
    println!(
        "\x20   ALL ROWS       observable {observable}   in-class {in_class} ({})   \
         **FUSED {fused}** ({})   residue {residue} ({})\n\
         \x20     FUSED = strict REFUSED, relaxed ADMITTED — the symbol-binding layer.\n\
         \x20     residue = refused on BOTH sides: what this seam does NOT reach, i.e. the part \
         of \"symbol binding\" that is not symbol RESOLUTION.{}",
        pct(in_class, observable),
        pct(fused, observable),
        pct(residue, observable),
        if fused == 0 {
            "\n\x20     ** FUSED IS ZERO — THIS INSTRUMENT IS NOT MEASURING SYMBOL BINDING **"
        } else {
            ""
        }
    );
    // **THE CROSS-WALK IDENTITY** (#3288's second-derivation pattern): the
    // grammar-reached half of this walk's two refusal cells must add up to the
    // number `gap::decode` filed off `Decoded` in a different module.
    let (fg, rg) = (s("symbind-fused-grammar"), s("symbind-residue-grammar"));
    let gna = report.decode_total("decode-reach-grammar-not-admitted");
    println!(
        "\x20   SECOND DERIVATION — `symbind-fused-grammar` {fg} + `symbind-residue-grammar` \
         {rg} = {} against `decode-reach-grammar-not-admitted` {gna} (filed by \
         `gap::decode` off `Decoded`, code this module did not write); disagreement {}. Known \
         answer 0. **{gna} IS THE BASELINE, NOT ZERO** (#3582) — the I1 signal, and this \
         lane's, is the CHANGE in these populations and never their distance from 0.",
        fg + rg,
        (fg + rg).abs_diff(gna),
    );
    for (label, prefix, cap) in [
        (
            "WHICH REFUSAL a fused body carried (strict census key)",
            "symbind-fused|",
            12usize,
        ),
        (
            "WHICH GRAMMAR was underneath it (the relaxed census's accepted shape)",
            "symbind-fused-shape|",
            16,
        ),
        (
            "THE CROSS — refusal x grammar. **This is the \"one phenomenon or several\" answer**; \
             a key whose row spans many shapes is not naming one construct",
            "symbind-fused-cross|",
            24,
        ),
        (
            "WHICH SIDE of the binding was blind (`$blind$callee` / `$blind$data` in the relaxed \
             body). `neither` is an ANOMALY arm and is printed",
            "symbind-missing|",
            8,
        ),
        (
            "HOW MANY placeholder SITES per fused body — \"one symbol\" and \"many\" as two numbers",
            "symbind-missing-sites|",
            8,
        ),
        (
            "the fused population crossed with FRAME reach (the three strengths are NOT a chain, \
             #3582)",
            "symbind-fused-frame|",
            4,
        ),
        (
            "MANGLING CLASS of the refused function",
            "symbind-fused-mangling|",
            10,
        ),
        (
            "PER-TU CONCENTRATION (one bucket per TU — a bucket and not a max, because these \
             maps are SUMMED)",
            "symbind-tu-bucket|",
            8,
        ),
        (
            "THE RESIDUE, named on BOTH sides (strict key | relaxed key), grammar-reached rows \
             only. A count of disagreements that cannot be looked at is not a repair set",
            "symbind-residue|",
            12,
        ),
        (
            "…and WHAT THOSE RESIDUE BODIES ARE — the strict key x the body-dispatch arm that \
             claimed them (`FnCensus::dispatch`, decode-only). A key naming a CALLEE whose rows \
             sit under an arm that reads no callee is a MISNAMED key, not a missing symbol",
            "symbind-residue-dispatch|",
            16,
        ),
        (
            "…and their FRAME CLASS. **`calls-0` under a `callee-unresolved-*` key is a body \
             that issues NO CALL AT ALL** — the sharpest available statement that the key is \
             misnamed for that row",
            "symbind-residue-frame|",
            12,
        ),
        (
            "the FUSED population's dispatch arm, printed beside the residue's so the two \
             halves are comparable",
            "symbind-fused-dispatch|",
            12,
        ),
        (
            "…and the fused population's frame class",
            "symbind-fused-frameclass|",
            4,
        ),
    ] {
        let rows = report.symbind_rows_by_name(prefix);
        println!(
            "\x20   {label} — {} distinct, sorted by NAME (never by mass, #3505):",
            rows.len()
        );
        if rows.is_empty() {
            println!("\x20     (none)");
        }
        for (k, n) in rows.iter().take(cap) {
            println!("\x20     {n:>8}  {k}");
        }
        if rows.len() > cap {
            println!("\x20     … and {} more", rows.len() - cap);
        }
    }
    println!(
        "\x20   EMITTED CROSS  named {}  unnamed {}  model-reached {}  relaxed-gate-refused {} \
         — for a body c2 never emits, \"in class\" is a parser-only claim no byte compare has \
         ever graded or ever can (`FnCensus::emit_name`).\n\
         \x20   BLIND SITES    callee {}  data {} (site totals over the fused population, \
         {} TUs of {} carry at least one fused row)",
        s("symbind-fused-named"),
        s("symbind-fused-unnamed"),
        s("symbind-fused-model"),
        s("symbind-fused-relaxed-gate-refused"),
        s("symbind-blind-callee-sites"),
        s("symbind-blind-data-sites"),
        s("symbind-tus-any"),
        s("symbind-tus-scanned"),
    );
    println!(
        "\x20   controls (all known answer 0): monotonicity-broken {mono}  partition-broken {}  \
         population-broken {}  census-desync {}  placeholder-none {}\n\
         \x20     `monotonicity-broken` = a row the STRICT census admits and the RELAXED one \
         refuses. It is NOT true by construction — it is the claim that supplying a name only \
         ever widens — and its failure is EXECUTED in a unit test.\n\
         \x20     `placeholder-none` = a FUSED row with no `$blind$*` anywhere the public \
         accessors reach. Nonzero is a FINDING (this module's account of the seam is \
         incomplete by exactly that many bodies), not an alarm. **THE IDENTITY OF THE MISSING \
         SYMBOL IS NOT VISIBLE HERE AT ALL**: `FnCensus` publishes no such field and `c2-il` is \
         read, never written, by this lane. Owed, not smuggled.",
        s("symbind-partition-broken"),
        s("symbind-population-broken"),
        s("symbind-census-desync"),
        s(super::symbind::PLACEHOLDER_NONE_KEY),
    );
}

fn render_plan(report: &GapReport) {
    let ctl = report.plan_control();
    println!(
        "\n\x20 OBJECT PLAN (lane w-objplan) — everything about the output obj that is \
         INDEPENDENT OF THE INSTRUCTION BYTES, graded on every TU. TWO INDEPENDENT PRODUCERS: \
         `observe` reads the REFERENCE obj (ground truth); `c2_core::plan::predict` computes the \
         port's plan FROM IL WITHOUT EMITTING. A grade taken over the port's own obj would be \
         VACUOUS on the matched TUs (there its bytes ARE the reference's) and UNDEFINED on the \
         ones it refuses. **THIS IS AN INSTRUMENT AND NEVER A GATE: `exact` here is NECESSARY \
         but NOT SUFFICIENT for `match` — a TU can be plan-exact and mismatch on every byte.**"
    );

    // --- the named control, FIRST -----------------------------------------
    println!(
        "\x20   CONTROL (pinned BY NAME in docs/plan/CONTROL_TUS.txt — a control pinned by COUNT \
         passes in an unprovisioned worktree the moment the count matches, #3219/#3231): \
         {} pinned, {} `match` TUs found this scan, {} of the pinned present in this scan, \
         {} entered, {} left.",
        ctl.pinned,
        ctl.found,
        ctl.present,
        ctl.diff_entered.len(),
        ctl.diff_left.len()
    );
    if !ctl.diff_entered.is_empty() || !ctl.diff_left.is_empty() {
        println!(
            "\x20     SET DIFFERENCE — the match set moved under the pin. This is a finding about \
             the TREE or the WORKLOAD STAMP and is reported BEFORE any number below it."
        );
        for n in &ctl.diff_entered {
            println!("\x20       entered: {n}");
        }
        for n in &ctl.diff_left {
            println!("\x20       left:    {n}");
        }
    }
    println!(
        "\x20     {} of {} pinned TUs are `exact` on every shipped component; {} shortfall \
         cell(s) — {} `differs` and {} `unknown`. **THE REGISTERED RULE (prereg §3) IS THAT A \
         COMPONENT WHOSE CONTROL IS RED SHIPS AS `unknown`, NEVER AS `differs`, AND IT IS \
         APPLIED UNIFORMLY**: `emitset-order` differed on 12 of 26 and `emitset-members` on 2 \
         of 26, so BOTH are withdrawn and both predictors are published as characterization \
         instead. The first version of this instrument shipped the second one on an \
         unregistered \"12 is too many, 2 is fine\" and then printed its own `2 differs` on \
         every scan. (A count, not a status; `0 of 0` would say so rather than printing \
         nothing.)",
        ctl.exact_rows,
        ctl.present,
        ctl.shortfall.len(),
        ctl.differ_cells,
        ctl.unknown_cells
    );
    // **THE CONTROL'S OWN SIZE.** Printed on the line after the headline and
    // never further away: "24 of 26 exact" and "24 empty comparisons" are the
    // same sentence without it, which is this instrument's own §2.3 finding
    // arriving on its control instead of on its curve.
    println!(
        "\x20     CONTROL SIZE — the pinned TUs carry {} emitted name(s) between them; {} of \
         {} compare an EMPTY reference set (where every pure manifest agrees and nothing can \
         be detected) and {} compare a set of >= 2 names (the only cells where a membership \
         OR an ordering error can show). WHAT THIS CONTROL CAN DETECT: a component claiming to \
         disagree on a TU the byte judge called equal; a predictor naming a function c2 did \
         not emit; the pinned set moving under the reader. WHAT IT CANNOT: anything on the \
         empty cells; anything about ORDER where the set has <= 1 name; an extractor that \
         collapsed to the empty set — which is why the SIZE is published rather than inferred, \
         so that failure reads as this number falling and not as the exact count rising.",
        ctl.obs_size,
        ctl.obs_empty_tus,
        ctl.present,
        ctl.substantive_tus
    );
    for (src, component, v, why) in &ctl.shortfall {
        println!("\x20       {src}  {component} = {}  — {why}", v.label());
    }

    // --- the curve ---------------------------------------------------------
    let distinct = report.plan_distinct();
    let rows = report.plan_rows();
    println!(
        "\x20   PER COMPONENT over {} graded TUs — `observable` (the reference decoded) ⊇ \
         `known` (the port also answered) ⊇ `exact`. `differs` is DERIVED here, never by the \
         reader (board #213). `distinct` counts the DISTINCT observed values across the \
         workload: a component at 1 is FREE — agreeing with a constant is not progress.",
        rows.len()
    );
    for (i, k) in super::plan::PLAN_KEYS.iter().enumerate() {
        let v = |r: &super::plan::PlanRow| r.verdicts[i];
        let observable = rows
            .iter()
            .filter(|r| v(r) != super::plan::PlanVerdict::Unobservable)
            .count();
        let exact = rows
            .iter()
            .filter(|r| v(r) == super::plan::PlanVerdict::Exact)
            .count();
        let known = rows
            .iter()
            .filter(|r| {
                matches!(
                    v(r),
                    super::plan::PlanVerdict::Exact | super::plan::PlanVerdict::Differs
                )
            })
            .count();
        let d = distinct.get(k.component).copied().unwrap_or(0);
        println!(
            "\x20     {:<18} observable {:>4}  known {:>4}  exact {:>4}  differs {:>4}  \
             distinct {:>4}{}",
            k.component,
            observable,
            known,
            exact,
            known - exact,
            d,
            if d == 1 { "   [FREE — one value across the workload; excluded from any headline]" } else { "" }
        );
    }

    // --- why the port did not look ------------------------------------------
    //
    // The `Unknown` histogram IS the ranking of stages that owe work, and it is
    // the entire mitigation for #3237: an instrument that returns 0 because it
    // did not look must say so, by name.
    let mut why: std::collections::BTreeMap<&str, usize> = Default::default();
    for r in report.graded() {
        for (c, reason) in &r.plan.reasons {
            if r.plan.verdicts.get(c) == Some(&super::plan::PlanVerdict::Unknown) {
                *why.entry(reason.as_str()).or_insert(0) += 1;
            }
        }
    }
    if why.is_empty() {
        println!("\x20   no component read `unknown` on any graded TU.");
    } else {
        println!(
            "\x20   WHY THE PORT DID NOT LOOK — component-verdicts of `unknown`, by the reason \
             the predictor named. This histogram IS the ranking of which stage owes the work, \
             and naming it is the whole mitigation for #3237:"
        );
        for (r, n) in &why {
            println!("\x20     {r:<32} {n:>5}");
        }
    }

    // --- the seed, sized ----------------------------------------------------
    let subset = rows.iter().filter(|r| r.subset == Some(true)).count();
    let with_both = rows.iter().filter(|r| r.subset.is_some()).count();
    let extra: usize = rows.iter().filter_map(|r| r.extra).sum();
    let missing: usize = rows.iter().filter_map(|r| r.missing).sum();
    let pred_size: usize = rows.iter().filter_map(|r| r.pred_size).sum();
    let obs_size: usize = rows.iter().filter_map(|r| r.obs_size).sum();
    let seed_empty = rows.iter().filter(|r| r.pred_size == Some(0)).count();
    let obs_size_known: usize = rows
        .iter()
        .filter(|r| r.pred_size.is_some())
        .filter_map(|r| r.obs_size)
        .sum();
    let seed_exact = rows
        .iter()
        .filter(|r| r.extra == Some(0) && r.missing == Some(0))
        .count();
    let seed_exact_sub = rows
        .iter()
        .filter(|r| r.extra == Some(0) && r.missing == Some(0) && r.obs_size.unwrap_or(0) > 0)
        .count();
    let glorder_known = rows.iter().filter(|r| r.glorder.is_some()).count();
    let glorder_agrees = rows.iter().filter(|r| r.glorder == Some(true)).count();
    println!(
        "\x20   EMIT-SET SEED (`docs/whitebox/C2_MAP.md` §3E, flag word `sym+0x4c` bit 0x20, \
         `test dl,0x20` at 0x10b7f16e) — the emitted set is the SEEDED set CLOSED under \
         \"referenced by an already-emitted function\", and §3E's own warning is that a port \
         using the seed ALONE will OVER-DELETE on real TUs. So the seed is expected to be a \
         SUBSET and the gap is the closure's size, measured rather than argued: {subset} of \
         {with_both} TUs where both sides answered have seed ⊆ emitted; {extra} over-claimed \
         name(s) in total (a nonzero here is a finding about the BIT, not about the port); \
         {missing} emitted name(s) the seed does not carry (that is the CLOSURE's work). \
         **THE CLAIMANT'S OWN SIZE, because a containment claim without it is unfalsifiable in \
         the flattering direction — the empty set is a subset of everything:** the seed names \
         {pred_size} function(s) against c2's {obs_size_known} emitted over the SAME \
         {with_both} TUs ({obs_size} over all {} graded, which is the figure that reconciles \
         with `fnbyte-denominator`), and is EMPTY on {seed_empty} of the {with_both}. \
         **THE SEED IS NOT A COMPONENT** — its control differs on 2 of the 26 pinned TUs, so \
         under prereg §3 it does not ship and every figure on this line is CHARACTERIZATION. \
         It equals c2's emitted set on {seed_exact} TUs, of which only \
         **{seed_exact_sub} are substantive** — the rest compare the empty set to the empty \
         set, which every pure predictor gets right.",
        rows.len()
    );
    println!(
        "\x20   `.gl` RECORD ORDER IS NOT COMDAT ORDER — REFUTED BY THE NAMED CONTROL, and kept \
         as a CHARACTERIZATION number rather than as a component: it agrees on {glorder_agrees} \
         of {glorder_known} TUs where both sides answered. This is the figure board #259's \
         `coff::order::plan_text_order` has to beat, and it did not exist before. It is NOT a \
         port curve and must not be read as one — `emitset-order` publishes `unknown` \
         everywhere, which is the honest state of a rule its own control killed."
    );

    // --- the observe-side inventory -----------------------------------------
    println!(
        "\x20   REFERENCE-SIDE INVENTORY over the graded TUs — NOT a curve. These are read off \
         real c2's objs and describe the population the un-conjuncted lanes must serve. They \
         re-derive figures this project has only ever CARRIED (`weak externals (675 TUs)`, \
         `COMDAT synthesis (450 TUs)` — quoted with no locator in ARCHITECTURE_PROPOSAL §1.2):"
    );
    for (k, n) in report.plan_observed() {
        println!("\x20     {k:<36} {n:>8}");
    }

    // --- the coverage probe on the reader the seed is read out of -----------
    //
    // MAJOR 3 of the review: `plan-glattr-names 28,107` against 162,146 emitted
    // has two stories, and the first version of this probe tested exactly one of
    // them (the uniform-zero mis-decode). These lines test the other.
    let obs = report.plan_observed();
    let g = |k: &str| obs.get(k).copied().unwrap_or(0);
    println!(
        "\x20   THE READER'S OWN COVERAGE — `gl_function_attrs` advances `p += 1` past any \
         position whose offset field is not framed, with NO refusal and NO counter, so a \
         systematically low hit rate looks exactly like a fact about `.gl` (#3237). It names \
         {} record(s), of which {} are functions c2 actually emitted. The ORTHOGONAL reader \
         `c2_il::mangled_names` — which does not use the framing at all — names {} symbol \
         run(s), of which {} are emitted functions, against {} emitted in total. **READ THE \
         TWO INTERSECTIONS TOGETHER:** if the run-based one reaches the emitted set and the \
         attr-based one does not, the shortfall is a fact about THIS SCANNER and not about `.gl`, \
         and every ceiling keyed off it has to be restated.",
        g("plan-glattr-names"),
        g("plan-glattr-in-emitset"),
        g("plan-glruns-names"),
        g("plan-glruns-in-emitset"),
        obs_size
    );
    println!(
        "\x20   THE ATTRIBUTE BYTE, BIT BY BIT — bit0 {} · bit1 {} · bit2 {} · bit3 {} · \
         bit4 {} · bit5 {} (the SEED) · bit6 {} (FN_FLAG_INLINABLE) · bit7 {} · byte==0x00 {}. \
         The second discriminator, and the one the first version of the probe did not take: \
         bit 6 at 99.99 % with zero `0x00` bytes rules out the UNIFORM-ZERO mis-decode and \
         nothing else — it cannot rule out a walk that landed on some other byte whose bit 6 \
         is usually set. A genuinely decoded field carries structure across its other six \
         bits; a mis-landed one tends to a near-constant value.",
        g("plan-glattr-bit0"),
        g("plan-glattr-bit1"),
        g("plan-glattr-bit2"),
        g("plan-glattr-bit3"),
        g("plan-glattr-bit4"),
        g("plan-glattr-bit5"),
        g("plan-glattr-bit6"),
        g("plan-glattr-bit7"),
        g("plan-glattr-zero")
    );
    println!(
        "\x20   AGREEMENT WITH THE INCUMBENT WALK, AT WORKLOAD SCALE (prereg tertiary, R2) — \
         `observe`'s emit set vs `text_comdat_functions` on {} TU(s): {} disagreement(s). \
         Known answer 0, and the POPULATION is printed beside it because an agreement over \
         zero TUs and a clean one look identical. Compared as SETS: the two walks are ordered \
         differently by construction (section table vs symbol table). The other three \
         accessors are still agreed on `tests/plan_agreement.rs`' three synthetic objs ONLY.",
        g("plan-agree-emitset-tus"),
        g("plan-agree-emitset-disagree")
    );

    let viol: usize = rows.iter().map(|r| r.violations).sum();
    let reached: usize = rows.iter().map(|r| r.checks).sum();
    println!(
        "\x20   CONTAINMENT CONTROL — `plan-bounds-violations` {viol} of {reached} check(s) \
         REACHED. A COUNT and not a status (STATUS trap 5) — **and the denominator is there \
         because the count itself had the defect the count exists to prevent**: three of its \
         four checks are unreachable while both components ship `unknown`, so a bare 0 could \
         not be told apart from not looking. Known answer 0: an ordered sequence cannot be \
         right while the set it orders is wrong, set equality implies containment (checked on \
         the characterization seed, where it is NOT true by construction), and no component \
         may grade a TU whose reference obj did not decode."
    );
}

/// **The known-answer control on the byte-fraction ranker** (board **#501**).
///
/// A `match` TU is byte-identical to `c2`'s obj, so the port demonstrably
/// produced a body for **every** `.text` byte in it: its byte fraction must read
/// **100 %**. That is the one population where the answer is known in advance,
/// and checking it is what stops the ranker from being a number nobody can
/// falsify — the numerator could silently stop crediting a whole `Selected`
/// variant and every frontier row would go on printing a plausible small
/// percentage.
///
/// **Printed as a count, never as a status** (`docs/GAPS.md` §7, STATUS trap 5):
/// `N of M matched TUs at 100 %`, with every shortfall named and its bytes
/// given. A run that graded zero matched TUs prints `0 of 0` and says so rather
/// than printing nothing.
///
/// **A shortfall here is not automatically a ranker defect.** The scan's own
/// emitted-census line already reports `ground truth VIOLATED: 2 emitted symbols
/// on the byte-exact TUs did not bind to an in-class row` — those land in
/// [`super::fnbytes::FnByte::Unbound`], which the numerator deliberately does not
/// credit, so they subtract here too. The control is read for its *movement*:
/// the named set is the thing to compare across trees.
fn render_byte_fraction_control(report: &GapReport) {
    let matched = report
        .results
        .iter()
        .filter(|r| r.class == TuClass::Match)
        .count();
    // Computed in `factors.rs` beside the ranking it controls, so the printed
    // block and the `gap-metric` keys cannot drift apart.
    let (full, nodenom, short) = report.byte_fraction_control();
    let unexplained = short.iter().filter(|(e, ..)| !*e).count();
    println!(
        "\x20 BYTE-FRACTION CONTROL (board #501) — a `match` TU is byte-identical to c2's obj, so \
         the port produced a body for every `.text` byte in it and its fraction MUST read 100%. \
         {full} of {matched} matched TUs do; {} fall short; {nodenom} have no `.text` denominator \
         (a TU that defines data and no functions — board #276's shape — which is a 100%-free \
         zero, not a failure).",
        short.len()
    );
    if short.is_empty() {
        println!("\x20   no shortfall. (A count, not a status: the {full} is the evidence.)");
    } else {
        println!(
            "\x20   SHORTFALL, by name, each classified — {} explained by factor E (whole-TU \
             emitter, per-function path blind by construction), {unexplained} UNEXPLAINED. \
             **The unexplained count is the one to watch; it must be 0.** Compare the SET across \
             trees, not its size. The FRONTIER is unaffected either way: it is defined as \
             `A and B and C and not (D or E)`, so no factor-E TU can appear in the ranking above.",
            short.len() - unexplained
        );
        for (e, r, n, d) in &short {
            println!(
                "\x20     {} ({n}/{d} bytes, {:.1}%)  {}",
                r.src,
                100.0 * *n as f64 / *d as f64,
                if *e {
                    "[factor E — a WHOLE-TU recognizer emitted this; the per-function path \
                     cannot see it. EXPECTED.]"
                } else {
                    "[NOT factor E — UNEXPLAINED, and the ranker's numerator is the first \
                     suspect.]"
                }
            );
        }
    }
}

/// **The byte-fraction ranking of the FRONTIER** (lane `w-tu3`, board **#500**).
///
/// The FRONTIER block above ranks by *blocked function count*, which is board
/// **#198**'s standing complaint and has now mis-ranked a target twice: #269
/// counted refusals and could not see what was already emitted, and **#465**
/// counted already-emitted *functions* and could not see how much of the TU they
/// were. This block is the third unit, and the only one with an outcome behind
/// it — see [`byte_fraction`] for `mmio` 72.7 % by function against 16.8 % by
/// byte, and `xboxmem` 50 % / 54.5 % with the conversion.
///
/// **Every ratio prints its denominator**, and a TU with no `.text` bytes prints
/// `n/a` under a counted reason rather than 100 %. `exact` is the
/// oracle-graded floor under `accepted`.
///
/// **This is an instrument and not a gate.** It ranks; it licenses nothing.
fn render_byte_fraction_ranking(report: &GapReport) {
    // Computed in `factors.rs`; see `GapReport::frontier_byte_ranking`.
    let rows = report.frontier_byte_ranking();
    let no_den = rows.iter().filter(|(_, f)| f.is_none()).count();
    let zero = rows.iter().filter(|(_, f)| matches!(f, Some((0, _)))).count();
    println!(
        "\x20 FRONTIER BY `.text` BYTE FRACTION (board #500) — how much of each TU's `.text` the \
         port already produces a body for, BY BYTE. Board #465 counted FUNCTIONS and was refuted \
         by the TU registered to confirm it (mmio: 72.7% by function, 16.8% by byte, DECLINED; \
         xboxmem: 50.0% by function, 54.5% by byte, CONVERTED). n = 2 outcomes — this ranks, it \
         does not license. `exact` is the oracle-graded floor under `accepted`; a TU with no \
         `.text` bytes prints n/a, NEVER 100%. {} of {} frontier TUs have NO denominator, and \
         {zero} are at EXACTLY 0% — TUs where codegen breadth has not begun:",
        no_den,
        rows.len()
    );
    // **The REMAINING column is a FOURTH unit and it is printed, not chosen
    // between** (board #505). `total − accepted` is the PowerPC the port must
    // learn to write for this TU, and on the only two cells with outcomes it
    // agrees with the fraction — `xboxmem` 60 bytes remaining and converted,
    // `mmio` 316 and declined, a 5.3x margin next to the fraction's 3.2x. On the
    // CURRENT frontier the two units DISAGREE at the head: the fraction says
    // `mmio` (16.8%, 316 B remaining) and the remainder says `Primes.cpp` (0%,
    // 64 B remaining). Neither is validated at n = 2, and this project has now
    // been wrong about the unit twice (#269, #465). Printing both is the honest
    // state; picking one on this evidence would be the third mistake.
    println!(
        "\x20    accepted/total bytes    frac   exact  remain | src   (REMAIN = total - accepted \
         = the PowerPC still to write. A FOURTH unit, printed and NOT chosen between: it agrees \
         with `frac` on both outcome cells and DISAGREES with it at this frontier's head.)"
    );
    for (r, f) in &rows {
        match f {
            Some((n, d)) => println!(
                "\x20   {n:>8}/{d:<8} bytes  {:>5.1}%  {:>5}  {:>6} | {}",
                100.0 * *n as f64 / *d as f64,
                byte_fraction_exact(r),
                d - n,
                r.src
            ),
            None => println!(
                "\x20   {:>8}/{:<8} bytes  {:>6}  {:>5}  {:>6} | {}",
                "-", 0, "n/a", "-", "n/a", r.src
            ),
        }
    }
}

/// **The compiler-label channel, per TU** (lane `w-loop`, board **#742**) — is
/// the value of c2's label counter written into this TU's obj at all?
///
/// `$M<n>`/`$T<n>` short names are the only channel; `coff::plan_labels` mints
/// them for a **framed** function and for nothing else. Measured over 34
/// leaf-only probe TUs across 17 control-flow shapes: **zero** labels, 28 of
/// them carrying a backward branch, with the 17 leaf+framed controls minting a
/// triple 17 of 17 (`work/w-loop/loopcost.py --q2`).
///
/// Three readings and they are three, not two:
///
/// * `label-free` — the obj carries no `$M`/`$T`. A leaf loop charges the
///   counter `+1..+4` (measured, `docs/LABEL_COUNTER.md` §4.2) and
///   `plan_labels` charges 0, but **that error has nowhere to land here**.
/// * `labels N` — it carries N of them, so a wrong charge is wrong bytes in the
///   symbol table.
/// * `label ??` — the obj did not decode. **Not** a `label-free`.
///
/// **An instrument, never a licence.** It reads c2's own output, so it can say
/// which TUs the counter cannot hurt; it cannot say that any of them is
/// emittable, and every one of them is still gated on codegen that does not
/// exist.
fn label_channel(r: &TuResult) -> String {
    if r.emit.get("emit-label-unreadable").copied().unwrap_or(0) > 0 {
        return "label ??".into();
    }
    if r.emit.get("emit-label-readable").copied().unwrap_or(0) == 0 {
        // The key is written unconditionally once the obj is read, so its
        // absence means the obj was never read — which is a third answer and
        // is printed as one.
        return "label n/e".into();
    }
    match r.emit.get("emit-label-syms").copied().unwrap_or(0) {
        0 => "label-free".into(),
        n => format!("labels {n}"),
    }
}

/// The legend under the CFG screen, printed as counts rather than as prose so
/// a run in which nothing was measured is visible (`docs/GAPS.md` §7).
fn render_label_channel_legend(rows: &[(&TuResult, CfgReach)]) {
    let free = rows.iter().filter(|(r, _)| label_channel(r) == "label-free").count();
    // `needs_class` and NOT set membership of the bare string: once a lane
    // restricts `cflow-loop` (board #778) a partial miss is spelled
    // `cflow-loop!<key>`, and a bare-string test would stop counting those
    // silently — the count would fall and nothing would say why.
    let loopy = rows.iter().filter(|(_, v)| v.needs_class("cflow-loop")).count();
    let loop_free = rows
        .iter()
        .filter(|(r, v)| v.needs_class("cflow-loop") && label_channel(r) == "label-free")
        .count();
    println!(
        "\x20   LABEL CHANNEL (board #742) — `$M`/`$T` are the ONLY way the value of c2's \
         compiler-label counter reaches an obj, and `coff::plan_labels` mints them for a FRAMED \
         function and nothing else. A leaf loop charges that counter +1..+4 while `plan_labels` \
         charges 0 (17 seed-free cells, docs/LABEL_COUNTER.md §4.2), which is the whole stated \
         justification for `codegen::labels` invariant 4 refusing every BACKWARD branch. On a \
         `label-free` obj that error has nowhere to land — and that is SHIPPED rather than \
         observed: `IlFunction::label_slots` returns `None` for THREE of the five loop shapes the \
         port emits, so a TU pairing one of THOSE with a framed function still refuses. **Two are \
         lifted, both at a charge of 2 and both spelled `for`** (board #746's fence B): the \
         pointer-walk loop, measured against `fixtures/cpp/whash_loop_then_framed.cpp`'s own obj \
         (lane `w-fenceb`, the first time this counter was PAID rather than refused, boards \
         #746/#747/#3091, control `whash_ptr_walk_loop.cpp`), and the float array-walk loop, \
         measured against `fixtures/cpp/wblockir_float_walk_then_framed_neg.cpp`'s (lane \
         `w-slots`, control `wblockir_float_walk.cpp`). Both TUs are `match` at `/O1`. The three \
         that remain are the `while`-spelled chain walk, the `do/while` free-list constructor, \
         and the counted accumulate — whose charge moves across modes on a class accepting BOTH, \
         so it is the one arm that must NOT be taken. \
         {free} of {} frontier TUs are \
         label-free; of the {loopy} blocked on `cflow-loop`, {loop_free} are. NOT a licence — \
         every one is still gated on codegen that does not exist, and the counter is only the \
         FIRST of that TU's refusals.",
        rows.len()
    );
}

/// **THE CFG-REACHABILITY SCREEN, printed on every scan** (lane `w-tu4`, board
/// **#720**) — see [`GapReport::frontier_cfg_reachability`] for the definition
/// and for why no byte/function/refusal count can express it.
///
/// The three rankings above (#269 refusals, #465 functions, #500 bytes) all
/// measure *how much progress exists* on a TU. This measures something with a
/// different type: **whether the emitter can express the TU's blocked functions
/// at all.** A TU can be one 8-byte function from matching and still be
/// unreachable, because the 8 bytes are a loop and `Selected` has no variant
/// with a backward branch.
///
/// **Printed as a partition with every member named**, never as a score and
/// never as a status — the same discipline the byte-fraction control follows.
/// The `Unclassified` bucket is printed as its own row rather than folded into
/// either side, because "the census bailed before it could tell" is a third
/// answer and folding it would make an ignorance look like a verdict.
fn render_cfg_reachability(report: &GapReport) {
    let rows = report.frontier_cfg_reachability();
    let reach = rows.iter().filter(|(_, v)| v.is_reachable()).count();
    println!(
        "\x20 FRONTIER BY CFG REACHABILITY (board #720) — CAN THE EMITTER EXPRESS THIS TU AT ALL? \
         `Selected` covers FOUR control-flow shapes: straight-line, ONE two-arm conditional, \
         — since lane `w-hash`, board #761 — ONE loop, the pointer-walk accumulate of \
         `codegen::ptr_walk_loop`, and — since lane `w-cfgclass`, board #1630 — ONE `if`/`else` \
         with a join whose arms are calls, `codegen::if_call_join`. **The last two are each a \
         transcription of a single function class, not a lowering of its CFG class**: twenty \
         words, two immediate fields, `/O1` only. Every other loop shape and every other \
         `cflow-if-n` shape still has no representation at all — the `if`/`else` class takes 2 of \
         the frontier's 11 `cflow-if-n` functions and the loop 1 of its 21 `cflow-loop` ones. \
         This line read `no variant encodes a backward branch, so NO loop of any kind has a \
         representation` until `Sort.cpp` converted, and `THREE ... shapes` until \
         `negate_test.cpp` did; each correction is here rather than beside the old claim. \
         **A shape being in this list does NOT put its class in `PORT_CFG_CLASSES`** — the two \
         converted TUs left the frontier by MATCHING, so neither appears in the count below, \
         which is why that count can stand still while this sentence grows. This is not a \
         quantity of \
         progress like #269/#465/#500 — a TU can be one 8-byte function from matching and be \
         unreachable because those 8 bytes are a loop. INSTRUMENT, never a gate. {reach} of {} \
         frontier TUs are reachable:",
        rows.len()
    );
    for (r, v) in &rows {
        println!(
            "\x20   {:>3} blocked | {:<50} | {:<10} | {}",
            r.fn_blockers.values().sum::<usize>(),
            r.src,
            label_channel(r),
            v.label()
        );
    }
    render_label_channel_legend(&rows);
    // The control is printed whether or not it passes, and an ABSENT control
    // prints as absent rather than as a pass — `cfg_reach_control` returns
    // `None` for a scan whose list does not contain the TU.
    const CONTROL: &str = "src/xdk/nuispeech/xboxmem.cpp";
    match report.cfg_reach_control(CONTROL) {
        Some(true) => println!(
            "\x20   CONTROL {CONTROL}: PASS — the one TU ever converted from codegen breadth \
             carries only port CFG classes (measured: cflow-if-1 x3 + cflow-straight x1)."
        ),
        Some(false) => println!(
            "\x20   CONTROL {CONTROL}: **FAIL** — a matching TU carries a CFG class outside the \
             port's list, so PORT_CFG_CLASSES is wrong and every row above is suspect."
        ),
        None => println!(
            "\x20   CONTROL {CONTROL}: absent from this scan's list — NOT a pass, not evaluated."
        ),
    }
    render_cfg_subclass(report);
}

/// **THE CODEGEN COLUMN, PRINTED ON EVERY SCAN** (lane `w-column`, board
/// **#1474**) — see [`GapReport::frontier_codegen`] and
/// [`super::factors::FrontierCodegen`].
///
/// Board **#1463** published `NO COLUMN` in the codegen cell of all sixteen
/// frontier rows; **#1464** proved the driver had never had one to lose. This
/// block is the column, and its headline row is deliberately the one that says
/// *how much cannot be measured* — because on this frontier that is almost all
/// of it, and a table that printed only the measurable part would read as
/// though the frontier were nearly done.
fn render_frontier_codegen(report: &GapReport) {
    let rows = report.frontier_codegen();
    if rows.is_empty() {
        println!(
            "\x20 FRONTIER BY CODEGEN (board #1474): the frontier is EMPTY on this scan, so \
             there is no column. Not a pass — not evaluated."
        );
        return;
    }
    let sum = |f: fn(&super::factors::FrontierCodegen) -> usize| -> usize {
        rows.iter().map(|(_, c)| f(c)).sum()
    };
    let (den, exact, wrong, cgref, reader, ungraded) = (
        sum(|c| c.denominator),
        sum(|c| c.exact),
        sum(|c| c.wrong),
        sum(|c| c.cg_refused),
        sum(|c| c.reader),
        sum(|c| c.ungraded),
    );
    println!(
        "\x20 FRONTIER BY CODEGEN (board #1474) — THE COLUMN #1463 PRINTED AS `NO COLUMN`, read \
         off the judge's own per-function predicate (FBM) instead of off a reader ladder. \
         `wrong` is the reader accepting a body, the emitter LOWERING it, and the bytes or \
         relocations DIFFERING: the only positive codegen price this project can measure per \
         function. `cg-ref` is the reader accepting and the emitter DECLINING — read \
         `fnbytes::Decline`'s doc before sizing anything off it, because three of its four \
         stages are ZERO BY CONSTRUCTION while acceptance lives in the IL parser. **`reader` is \
         the hole**: the IL parser refused, so no codegen question was asked and none CAN be — \
         there is no IlFunction to hand to `select_function`. A TU's true codegen distance is \
         `wrong + cg-ref` PLUS an unknown amount hiding in `reader`, so every positive number \
         below is a LOWER BOUND OF UNKNOWN TIGHTNESS and never a price. {} frontier TUs, {den} \
         emitted functions: {reader} behind the reader ({}%), {} measurable, {exact} already \
         byte-exact.",
        rows.len(),
        if den == 0 { 0 } else { reader * 100 / den },
        wrong + cgref,
    );
    println!(
        "\x20   {:>4} {:>5} {:>5} {:>6} {:>6} {:>7} | {}",
        "den", "exact", "wrong", "cg-ref", "reader", "ungrade", "src"
    );
    for (r, c) in &rows {
        println!(
            "\x20   {:>4} {:>5} {:>5} {:>6} {:>6} {:>7} | {}{}",
            c.denominator,
            c.exact,
            c.wrong,
            c.cg_refused,
            c.reader,
            c.ungraded,
            r.src,
            if c.partition_broken() { "  **PARTITION BROKEN**" } else { "" }
        );
    }
    let broken = rows.iter().filter(|(_, c)| c.partition_broken()).count();
    println!(
        "\x20   TOTAL {den} = exact {exact} + wrong {wrong} + cg-ref {cgref} + reader {reader} \
         + ungraded {ungraded};  partition-broken {broken} (target 0)."
    );
    // The vacuity statement, printed from the numbers rather than asserted, and
    // printed in BOTH directions so a future frontier with real codegen debt
    // does not keep reading the caveat that fits today's.
    if wrong + cgref == 0 {
        println!(
            "\x20   **THE MEASURABLE CODEGEN PRICE OF THIS FRONTIER IS ZERO, AND THAT IS NOT \
             `THE CODEGEN WORK IS DONE`.** Every frontier function the reader accepts, the port \
             already emits correctly; all of the remaining distance is behind a reader refusal, \
             where this instrument cannot follow. Board #1464's finding, in the affirmative: \
             the codegen column exists and on this population it is EMPTY."
        );
    } else {
        println!(
            "\x20   {} frontier function(s) are a MEASURED codegen defect — the reader accepts \
             them and the port does not produce c2's bytes. These are the only frontier codegen \
             numbers on this board that are not hand-counts.",
            wrong + cgref
        );
    }
}

/// **THE SUB-CLASS MECHANISM'S OWN INSTRUMENT** (lane `w-subclass`, board
/// **#778**) — the narrowing bracket and the ledger, printed under the screen.
///
/// #778 was filed because `PORT_CFG_CLASSES` could hold only a wholesale claim,
/// so a lane with genuine *partial* coverage of a CFG class had to either
/// over-claim or record nothing. Two lanes chose to record nothing. The list now
/// holds `CfgClass { class, sub: Whole | Keys(&[…]) }`, and the danger a
/// restriction introduces is the opposite of the one it fixes: a sub-class
/// predicate that is **more permissive** than the flat list would let a lane
/// report coverage it does not have.
///
/// So the mechanism prints its own bracket every scan, from the live frontier:
/// `⊥` (nothing admitted) `⊆` `shipped` `⊆` `⊤` (every class admitted), plus
/// `enumerated` — the shipped list re-expressed as explicit key sets, which must
/// reproduce `shipped` TU for TU. **Sets by name, not counts**: `|⊥| ≤ |shipped|`
/// is satisfied by swapping one TU for another, so counts cannot tell nesting
/// from coincidence.
fn render_cfg_subclass(report: &GapReport) {
    let b = report.cfg_reach_bounds();
    println!(
        "\x20   SUB-CLASS NARROWING (board #778) — the screen can now hold a PARTIAL claim \
         (`CfgClass{{class, Whole | Keys(&[..])}}`), and this is the bracket that keeps a \
         restriction NARROWER OR EQUAL rather than wider. `admits` is `class == class && <sub>`, \
         so `Keys` only ever CONJOINS: a restriction can remove an admitted (class,key) pair and \
         never add one. That is algebra; these four numbers are the measurement, re-derived from \
         this scan's own frontier. BOTTOM {} ⊆ ENUMERATED {} == SHIPPED {} ⊆ TOP {}, of {} \
         frontier TUs ({} classes in TOP, {} (class,key) pairs enumerated).",
        b.bottom.len(),
        b.enumerated.len(),
        b.shipped.len(),
        b.top.len(),
        b.frontier,
        b.top_classes.len(),
        b.enumerated_keys,
    );
    println!(
        "\x20     BOTTOM=every entry restricted to NO keys (must be 0 — it is the live exercise \
         of the `Keys` path and the detector for a matcher that ignores its key); \
         ENUMERATED=every entry rewritten as the exact key set this scan observed (must equal \
         SHIPPED); TOP=every class the frontier mentions, wholesale — a HYPOTHETICAL the port has \
         no claim to, printed so the refusal has a size: {} of the {} frontier TUs are held back \
         by CFG class alone.",
        b.top.len().saturating_sub(b.shipped.len()),
        b.frontier,
    );
    println!("\x20     SHIPPED reachable: [{}]", b.shipped.join(", "));
    let v = b.violations();
    if v.is_empty() {
        println!(
            "\x20     NESTING: PASS — 0 violations over the 3 checks (BOTTOM⊆SHIPPED⊆TOP, \
             ENUMERATED==SHIPPED), taken as SETS by name."
        );
    } else {
        for line in &v {
            println!("\x20     NESTING: **FAIL** — {line}");
        }
    }
    // The ledger. Printed as counts per entry so a claim that does nothing is
    // visible; `Whole` rows cross-check nothing and say so rather than PASS.
    let led = report.cfg_subclass_ledger();
    let restricted = led.iter().filter(|r| r.listed.is_some()).count();
    let unwitnessed: usize = led.iter().map(|r| r.unwitnessed.len()).sum();
    let intruders: usize = led.iter().filter_map(|r| r.intruders.as_ref()).map(|v| v.len()).sum();
    println!(
        "\x20     LEDGER — {} entries, {restricted} of them RESTRICTED. A restricted entry is a \
         claim about named census keys, and two ways it goes quietly wrong are counted here: a \
         listed key no scan witnesses (a claim doing nothing — trap 5 with the claim still on the \
         page) and a key `admits` accepts that the entry does not list (the matcher and the \
         declaration disagreeing, which is what an exact→prefix slip looks like). unwitnessed \
         {unwitnessed}, intruders {intruders}.",
        led.len()
    );
    for r in &led {
        let form = match r.listed {
            None => "WHOLE".to_string(),
            Some(n) => format!("KEYS {n}"),
        };
        let cross = match &r.intruders {
            // No declaration to compare against: NOT a pass.
            None => "cross-check n/a (whole class — no declaration to compare)".to_string(),
            Some(x) if x.is_empty() => "cross-check PASS (0 intruders)".to_string(),
            Some(x) => format!("cross-check **FAIL**: admits {} it does not list: {}", x.len(), x.join(", ")),
        };
        println!(
            "\x20       {:<32} | {:<8} | {:>4} keys observed, {:>4} admitted | {} | unwitnessed {}",
            r.class,
            form,
            r.observed_keys,
            r.admitted_keys,
            cross,
            r.unwitnessed.len()
        );
    }
    println!(
        "\x20     NOT A LICENCE, and #778 is closed as a MECHANISM only: `cflow-loop` did NOT \
         enter the list in the lane that built this. The list is still a hand-maintained mirror \
         of a `c2-core` enum; what changed is that a lane which has measured part of a class can \
         now write down which part, and the reachability figure it produces is bracketed above."
    );
}

/// **The Phase 7 factorization, printed on every scan** (`docs/ROADMAP.md`
/// §10.19 and §10.21, boards #160 and #179).
///
/// Printed from here rather than from `main.rs` for the same reason the splitter
/// block above is: the predicates live in this file, and a report assembled
/// somewhere else is a second place the definitions can drift.
///
/// **Everything here is a count.** There is no "factorization OK" line: a joint
/// that reproduced nothing would print zeros against a nonzero match count,
/// which is visible, where a status would not be (`docs/GAPS.md` §7).
pub(super) fn print_factorization(report: &GapReport) {
    let graded = report.graded().count();
    let [a, b, c, d, e, a_lo, bc, abc, abcd, joint] = report.factor_counts();
    let (bad, match_tus) = report.factor_control_on_match_tus();
    let matched: Vec<&str> = report
        .results
        .iter()
        .filter(|r| r.class == TuClass::Match)
        .map(|r| r.src.as_str())
        .collect();
    let all = report.factor_all_tus();
    let abcd_tus = report.factor_abcd_tus();
    let vocab = report.section_vocabulary();
    let unreadable = report.emit_total("emit-sec-unreadable");
    let readable = report.emit_total("emit-sec-readable");

    println!(
        "\nPHASE 7 FACTORS (ROADMAP §10.19/§10.21, boards #160/#179) — the model is \
         A and B and C and (D or E), over {graded} graded TUs\n\
         \x20 A  emit set reachable   `.ex` segments == obj `.text` COMDATs   {a:>5}  \
         (gate-anchored `4F 1F`; {a_lo} on the census's `4C 4F 11` anchor)\n\
         \x20 B  binding complete     every emitted symbol binds              {b:>5}\n\
         \x20 C  section shape        obj sections subset of the writer's {}   {c:>5}\n\
         \x20 D  per-fn acceptance    every emitted COMDAT in class           {d:>5}  \
         ({} of them emit nothing at all)\n\
         \x20 E  whole-TU acceptance  a REGISTERED whole-TU recognizer takes it {e:>3}  \
         ({} in the registry)\n\
         \x20   D and E are the two readings of ONE question — does the port have an accepted \
         route to this TU's contents. Neither is necessary alone (measured: D fails on the \
         whole-TU matches, E on the per-function ones); the DISJUNCTION is the term.\n\
         \x20 B and C jointly (the near-term ceiling, measured per TU — NOT a product of \
         marginals, §8.6): {bc}\n\
         \x20 A and B and C: {abc}   |   A and B and C and D (§10.19's original, refuted): \
         {abcd}   |   A and B and C and (D or E): {joint}   |   TUs the differential graded \
         `match`: {}\n\
         \x20 section headers: {readable} objs read, {unreadable} did not decode (outside C, \
         fail-closed)",
        PORT_WRITER_SECTIONS.len(),
        report.emit_total("emit-class-empty"),
        WHOLE_TU_RECOGNIZERS.len(),
        matched.len(),
    );
    // Per-recognizer marginals. A registry entry that never fires and one that
    // was never added are the same `|E|` and very different facts.
    for (name, n) in report.whole_tu_marginals() {
        println!("\x20   whole-TU recognizer `{name}` accepts {n} graded TU(s)");
    }
    // The refuted conjunction's own set, printed only when it differs from the
    // model's. This is the DELTA the fifth term accounts for, by name — §10.19's
    // claim is refuted by exactly these TUs and a reader should not have to
    // reconstruct which.
    if abcd_tus != all {
        let only_new: Vec<&&str> = all.iter().filter(|s| !abcd_tus.contains(s)).collect();
        println!(
            "\x20 the fifth term accounts for {} TU(s) that A and B and C and D misses: {:?}",
            only_new.len(),
            only_new
        );
    }
    // The set identity, by name. §10.19's claim is that the joint IS the match
    // set; two sets of the same size that differ by a swap would read as equal
    // if this printed only counts.
    if all == matched {
        println!(
            "\x20 the joint is EXACTLY the match set ({} TUs, by name): {}",
            all.len(),
            all.join(", ")
        );
    } else {
        let only_joint: Vec<&&str> = all.iter().filter(|s| !matched.contains(s)).collect();
        let only_match: Vec<&&str> = matched.iter().filter(|s| !all.contains(s)).collect();
        println!(
            "\x20 the joint is NOT the match set — {} in the joint only ({:?}), {} matching but \
             outside some factor ({:?}). The second list is the ALARM: every factor is meant \
             to be NECESSARY, so a matching TU outside one voids the bound it carries.",
            only_joint.len(),
            only_joint,
            only_match.len(),
            only_match,
        );
    }
    // **The known-answer control.** Split into the terms that must be zero and
    // the two diagnostics that must not be required to be — see
    // `factor_control_on_match_tus` for why moving D out of the first group is a
    // repair of the model and not a relaxation of the control.
    println!(
        "\x20 known-answer control — matching TUs failing each NECESSARY term (all must be 0, \
         over {match_tus} matching TUs): A {} B {} C {} D-or-E {}\n\
         \x20   diagnostics, NOT required to be 0 (neither disjunct is necessary alone): \
         D {} E {}",
        bad[0], bad[1], bad[2], bad[5], bad[3], bad[4]
    );
    // **W-R1c measured that D alone is not necessary; board #179 says what the
    // necessary term is instead.** §10.19's claim was that A∧B∧C∧D is exactly the
    // observed match set. That was measured when `PortC2` had exactly ONE
    // acceptance path, the per-function one, and D's proxy for "the port can emit
    // this" is the per-function census verdict. A `??__E` dynamic-initializer TU
    // is emitted by a **whole-TU** path (`IlBundle::dyninit_tu`), so its thunk is
    // byte-exact in the obj and out of class in the census at the same time —
    // both statements true, about different questions.
    //
    // The fix is a fifth term, E, disjoined onto D. It is NOT a widening of D:
    // D still reads exactly the per-function census, so the census/gate symmetry
    // `census.rs` maintains on purpose (tracked by the `census/gate disagreement`
    // line above) is untouched. And D's own violation count is still printed,
    // above, because a refutation whose evidence stops being reported is a claim
    // nobody can re-check.
    if bad[3] > 0 {
        println!(
            "\x20   D {} — the per-function census does not model the whole-TU emit path, so \
             §10.19's A∧B∧C∧D conjunction stays refuted by these TU(s). That is the finding \
             board #179 repairs with E, not a defect E hides.",
            bad[3]
        );
    }
    if bad[0] > 0 {
        println!(
            "\x20   A {} — a whole-TU DATA emitter needs no `.text`, so A's proxy (`.ex` \
             segments == `.text` COMDATs) is not necessary for a functionless-emit match: \
             `decomp_pch.cpp` matches with 1,242 segments and 0 `.text` (lane w-npos, the \
             provide-data-tu recognizer). A still bounds every PER-FUNCTION match and the \
             A-derived ceilings keep that scope; this count is the record that the scope \
             narrowed, printed the way D's refutation is, not a defect E hides.",
            bad[0]
        );
    }
    if bad[5] > 0 {
        println!(
            "\x20   ALARM: {} matching TU(s) are outside BOTH D and E. The port emitted \
             byte-exact bytes through a path no entry in WHOLE_TU_RECOGNIZERS models — \
             register it there, or the factorization's bound is void. This is the control \
             doing its job, not a number to tune away.",
            bad[5]
        );
    }
    // The vocabulary, in full. It is finite, and its size is the headline: it is
    // what makes C the one factor with a short route to closure.
    println!(
        "\x20 SECTION VOCABULARY — {} distinct names across the workload (objs carrying each):",
        vocab.len()
    );
    for (name, objs) in &vocab {
        let mine = if PORT_WRITER_SECTIONS.contains(&name.as_str()) {
            "writer"
        } else {
            "  ---"
        };
        println!("\x20   {objs:>5} objs  [{mine}]  {name}");
    }
    // The one ACTIONABLE row of the whole block: TUs no acceptance path covers.
    // Printed as a list with each one's distance, because a count would say "17"
    // and name nothing to work on.
    //
    // **Board #179 narrowed the membership from `¬D` to `¬(D∨E)`.** A TU a
    // whole-TU recognizer already accepts is not reachable by widening the
    // per-function class, so listing it here would advertise the wrong work.
    let frontier = report.factor_frontier();
    println!(
        "\x20 FRONTIER — {} graded TUs satisfy A and B and C, are NOT a match, and are outside \
         BOTH acceptance paths (not D, not E), so per-function codegen breadth is the whole \
         remaining distance (blocked emitted | emitted | src):",
        frontier.len()
    );
    for (r, blocked) in frontier.iter().take(40) {
        println!(
            "\x20   {blocked:>4} | {:>4} | {}",
            r.emit.get("emit-emitted").copied().unwrap_or(0),
            r.src
        );
    }
    if frontier.len() > 40 {
        println!("\x20   … and {} more", frontier.len() - 40);
    }
    render_byte_fraction_ranking(report);
    render_byte_fraction_control(report);
    render_cfg_reachability(report);
    render_frontier_codegen(report);
    // **What a perfect emit predicate is worth, stated as both of the
    // quantities board #213 conflated.** #213 published `+82` for both because
    // they coincided on that corpus; they are different questions and the
    // difference is printed by name whenever it is nonempty.
    let frontier_if_a = report.factor_frontier_if_a();
    let div = report.factor_projection_divergence();
    println!(
        "\x20 A PERFECT EMIT PREDICATE (board #213) — reachability, NOT conversions: every TU \
         below is still gated on codegen that does not exist.\n\
         \x20   as REACH:    B and C {bc} less A and B and C {abc} = +{}\n\
         \x20   as FRONTIER: frontier-if-A {frontier_if_a} less FRONTIER {} = +{}",
        bc.saturating_sub(abc),
        frontier.len(),
        frontier_if_a.saturating_sub(frontier.len()),
    );
    if div.is_empty() {
        println!(
            "\x20   the two agree ({}). They are still different questions — they coincide \
             only while no TU inside B and C fails A with an acceptance path already \
             covering it, which is a fact about this corpus.",
            bc.saturating_sub(abc)
        );
    } else {
        println!(
            "\x20   THE TWO DISAGREE by {} — #213's single `+82` is refuted on this tree. The \
             difference is exactly the TU(s) inside B and C that fail A and that the port \
             ALREADY accepts (D or E), so modelling the emit set makes them reachable \
             without adding them to the codegen frontier: {:?}",
            div.len(),
            div
        );
    }
    // …and the route: which name to teach next, by TUs brought into reach.
    let ladder = report.section_ladder();
    println!(
        "\x20 GREEDY LADDER — next section name to teach the writer, and the resulting C \
         ({} steps from {c} to {readable}):",
        ladder.len()
    );
    let mut prev = c;
    for (name, reach) in &ladder {
        println!("\x20   +{name:<12} C = {reach:>5}   (+{})", reach - prev);
        prev = *reach;
    }
    // **The same figures again, in a form a collector can take.** See
    // `GapReport::metrics` for why: C, `A∧B∧C` and the FRONTIER are printed by
    // every scan and are still hand-copied into `STATUS.md`, and all three went
    // stale twice on 2026-08-04. `B∧C` went stale by a *dependency* moving.
    // **PROGRESS MASS — printed apart from every correctness count, with the
    // disclaimer in the header rather than in a doc nobody re-reads.** The
    // byte-exact `match` line in the GAP REPORT below is the only correctness
    // number a scan produces; this block is the leading-indicator mass and can
    // NEVER substitute for it. See `GapReport::progress_mass` and
    // `docs/PROGRESS_METRIC.md` for the design and the two structural guards.
    match report.progress_mass() {
        Some(p) => {
            let g = p.graded as f64;
            println!(
                "\n\x20 PROGRESS MASS — a PROGRESS metric, NEVER a correctness signal \
                 (docs/PROGRESS_METRIC.md).\n\
                 \x20   The byte-exact differential is the SOLE judge; `match` below is the \
                 only correctness count.\n\
                 \x20   No term rewards emitted bytes: P counts proven preconditions (A, B, C \
                 against the reference obj)\n\
                 \x20   and honest acceptance (the emitted census). A TU graded `mismatch` \
                 contributes 0 to every\n\
                 \x20   numerator, so a wrong emit always scores below the refusal it \
                 replaced.\n\
                 \x20   P = mean(a, b, c, f) over {} graded TUs = {:.5}\n\
                 \x20     a  emit-set reachable (A)      {:>6}/{}  = {:.5}\n\
                 \x20     b  binding complete (B)        {:>6}/{}  = {:.5}\n\
                 \x20     c  section shape (C)           {:>6}/{}  = {:.5}\n\
                 \x20     f  emitted fns in class   {:>6}/{}  = {:.5}\n\
                 \x20   mismatch-zeroed TUs: {}",
                p.graded,
                p.value,
                p.a,
                p.graded,
                p.a as f64 / g,
                p.b,
                p.graded,
                p.b as f64 / g,
                p.c,
                p.graded,
                p.c as f64 / g,
                p.emitted_in_class,
                p.emitted_total,
                p.emitted_in_class as f64 / p.emitted_total as f64,
                p.mismatch_zeroed,
            );
        }
        None => {
            // Deliberately NOT 100 % and deliberately not silent: a scan that
            // graded nothing must make a positive claim of that fact.
            println!(
                "\n\x20 PROGRESS MASS: NO-RESULT — 0 graded TUs (or no emitted functions). \
                 A progress number over an empty scan is unrepresentable on purpose."
            );
        }
    }
    // **FUNCTION BYTE MATCH — the same judge, a finer unit.** Printed in its own
    // block, under the same disclaimer, because it is a PROGRESS instrument and
    // the `match` line below is still the only correctness count. See
    // `GapReport::fn_byte_match` and `docs/FUNCTION_BYTE_MATCH.md`.
    match report.fn_byte_match() {
        Some(f) => {
            let pct = |n: usize| 100.0 * n as f64 / f.denominator as f64;
            println!(
                "\n\x20 FUNCTION BYTE MATCH (FBM) — a PROGRESS instrument, NEVER a gate \
                 (docs/FUNCTION_BYTE_MATCH.md).\n\
                 \x20   The judge's own predicate — byte-identical to real c2 — asked per \
                 EMITTED FUNCTION instead of\n\
                 \x20   per TU. No partial credit: a wrong body scores exactly what a refusal \
                 scores, which is 0.\n\
                 \x20   The denominator is counted off c2's obj, so refusing more never \
                 shrinks it.\n\
                 \x20   FBM = ({} + {})/{} emitted functions byte-exact = {:.5}\n\
                 \x20     exact       {:>8}  ({:>5.2}%)   CREDITED — per-function route, bytes \
                 identical to c2's\n\
                 \x20     whole-TU    {:>8}  ({:>5.2}%)   CREDITED — on a TU the differential \
                 graded `match`; the judge certified the whole obj\n\
                 \x20     differs     {:>8}  ({:>5.2}%)   complete port body, bytes differ\n\
                 \x20     reloc-diff  {:>8}  ({:>5.2}%)   bytes IDENTICAL, RELOCATIONS DIFFER — \
                 two bodies branching to two different functions (board #884)\n\
                 \x20     reloc-unk   {:>8}  ({:>5.2}%)   bytes identical, reference relocation \
                 table did not decode — UNGRADED, never credited\n\
                 \x20     partial     {:>8}  ({:>5.2}%)   selected; the PORT's own /Gy \
                 composition declined the body (board #322 closed the harness's half)\n\
                 \x20     refused     {:>8}  ({:>5.2}%)   the port declines the function\n\
                 \x20     unbound     {:>8}  ({:>5.2}%)   no census row claims the symbol\n\
                 \x20     no-bytes    {:>8}  ({:>5.2}%)   COMDAT raw data did not decode\n\
                 \x20   objs unreadable (contribute NO denominator): {}   partition breaks \
                 (known answer 0): {}\n\
                 \x20   KNOWN-ANSWER CONTROL — per-function bodies that DIFFER on a TU the \
                 oracle graded `match` (must be 0): {}\n\
                 \x20   FIVE-ALARM CONTROL — byte-exact bodies whose RELOCATIONS differ on a \
                 `match` TU (must be 0): {}\n\
                 \x20   census/gate disagreement on EMITTED fns (the error term on the \
                 emitted census): {} TOTAL, of which {} are PARSER-EXPRESSIBLE \
                 (target 0 — board #139) and the rest are the post-lowering \
                 stages no parser clause can reach (`gy-shape`, `data-ref`, \
                 `inlined-callee`; see `gap-metric fnbyte-census-disagree-*`). \
                 The residue is the measured size of the emitted census's \
                 OVER-CLAIM, not an accounting convenience\n\
                 \x20   NOTE: the buckets partition the denominator by the PER-FUNCTION \
                 route alone; on a `match` TU the whole-obj verdict supersedes them.\n\
                 \x20   RELOC-EQ (lane w-relo, board #884 — `exact` now means bytes AND \
                 relocations):\n\
                 \x20     bytes-exact {:>8}   the OLD `exact`, i.e. what this instrument \
                 credited before relocations were graded\n\
                 \x20     graded      {:>8}   of those, how many got a relocation verdict; \
                 residue (ungraded) {}   reach breaks (known answer 0): {}\n\
                 \x20     of the CREDITED, {} carry at least one relocation — every one \
                 compared by offset, packed type and TARGET SYMBOL NAME.",
                f.exact,
                f.whole_tu,
                f.denominator,
                f.value,
                f.exact,
                pct(f.exact),
                f.whole_tu,
                pct(f.whole_tu),
                f.differs,
                pct(f.differs),
                f.reloc_differs,
                pct(f.reloc_differs),
                f.reloc_unknown,
                pct(f.reloc_unknown),
                f.partial,
                pct(f.partial),
                f.refused,
                pct(f.refused),
                f.unbound,
                pct(f.unbound),
                f.nobytes,
                pct(f.nobytes),
                f.obj_unreadable,
                f.partition_broken,
                f.match_tu_differs,
                f.match_tu_reloc_differs,
                f.census_disagree,
                f.census_disagree_expressible,
                f.exact_bytes,
                f.reloc_graded,
                f.reloc_unknown,
                f.reloc_partition_broken,
                f.exact_relocated,
            );
            // **The relocation families and their witnesses.** Printed as a
            // positive statement in BOTH directions: an empty families list says
            // so with its denominator, because an absent line is how absence
            // reads as success (this project's most-repeated defect).
            let fams = report.fn_byte_reloc_families();
            if fams.is_empty() {
                println!(
                    "\x20   reloc-differ families: NONE — {} byte-exact functions graded, \
                     {} of them relocating",
                    f.reloc_graded, f.exact_relocated
                );
            } else {
                println!(
                    "\x20   reloc-differ families (shape|kind|where->where|relation), \
                     most frequent first:"
                );
                for (k, n) in fams.iter().take(20) {
                    println!("\x20     {n:>6}  {k}");
                }
                // `blocked` means the port's own target is a body the parser
                // refused, so the chain question is not answerable — that is a
                // PRICE and it is named by production, never left as a residue.
                let blocked = report.fn_byte_reloc_blocked();
                if !blocked.is_empty() {
                    let rows: Vec<String> =
                        blocked.iter().take(12).map(|(k, n)| format!("{k} {n}")).collect();
                    println!(
                        "\x20   `blocked` families by the production that blocks the walk: {}",
                        rows.join(" · ")
                    );
                }
                let sigs = report.fn_byte_reloc_signatures();
                println!(
                    "\x20   reloc-differ signatures: {} distinct over {} functions; \
                     top rows with one example symbol each:",
                    sigs.len(),
                    f.reloc_differs
                );
                for (sig, n, ex) in sigs.iter().take(20) {
                    println!("\x20     {n:>6}  {sig}   e.g. {ex}");
                }
            }
            let (pw, rw, ew) = f.differ_words;
            if f.differs > 0 {
                // The objdiff-shaped number, confined to the class
                // `docs/PROGRESS_METRIC.md` §2 says it is legitimate on: bodies
                // the port DID produce and got wrong. It is a forensic aid on
                // the `differs` class and is aggregated into no headline —
                // raising it by emitting more nearly-right bodies raises FBM by
                // exactly nothing.
                println!(
                    "\x20   forensic (differs class ONLY, credited nowhere): {ew} of {rw} \
                     reference words positionally equal, port wrote {pw} words"
                );
            }
            // The under-report, by shape. This is a work list, not a defect
            // list: each row is a `Selected` variant whose body only the COFF
            // emitter can finish.
            let parts = report.fn_byte_partial_histogram();
            if !parts.is_empty() {
                let rows: Vec<String> =
                    parts.iter().map(|(k, n)| format!("{k} {n}")).collect();
                println!("\x20   partial by shape: {}", rows.join(" · "));
            } else {
                // **Printed as a positive statement, never as silence.** The
                // absent line used to mean "no under-report"; absence reading as
                // success is this project's most-repeated defect, so the empty
                // case says so with the denominator beside it.
                println!(
                    "\x20   partial by shape: NONE — every selected shape was reconstructed \
                     and graded ({} emitted functions)",
                    f.denominator
                );
            }
            // **The per-shape census** (board #322) — what the port selected,
            // crossed with what the judge said. `partial by shape` answered
            // "where is the alarm blind"; this answers "where is it now looking,
            // and what did it see", which is the line that has to be read before
            // `differs 0` means anything.
            let shapes = report.fn_byte_shape_census();
            if !shapes.is_empty() {
                let rows: Vec<String> = shapes
                    .iter()
                    .map(|(s, v, n)| format!("{s}/{v} {n}"))
                    .collect();
                println!("\x20   graded by shape × verdict: {}", rows.join(" · "));
            }
            // **WHOM THE BODY CALLS** (lane `w-drop3`, boards #984–#986). Every
            // line above this one compares `.text` bytes, and a `/Gy` branch
            // word cannot carry its callee: the placeholder displacement is
            // `-(offset of the word)` for every target alike, so a body that
            // calls the wrong function is byte-identical to one that calls the
            // right one. This row is the port's own call list against real c2's
            // `REL24` targets, by name.
            //
            // `disagree-exact` is the one to read: those are bodies FBM
            // **credits** whose relocations point somewhere else. Printed with
            // its denominator and printed at zero, because a control that
            // appears only when it fires is one whose silence means nothing.
            let ct_graded = report.emit_total("fnbyte-calltarget-graded");
            if ct_graded > 0 {
                println!(
                    "\x20   CALL TARGETS (name, not bytes): {} graded · {} agree · {} disagree \
                     ({} of them EXACT-and-wrong, {} differs) · {} by count, {} by name \
                     · {} ungraded",
                    ct_graded,
                    report.emit_total("fnbyte-calltarget-agree"),
                    report.emit_total("fnbyte-calltarget-disagree"),
                    report.emit_total("fnbyte-calltarget-disagree-exact"),
                    report.emit_total("fnbyte-calltarget-disagree-differs"),
                    report.emit_total("fnbyte-calltarget-disagree-count"),
                    report.emit_total("fnbyte-calltarget-disagree-name"),
                    report.emit_total("fnbyte-calltarget-ungraded"),
                );
                let ctw = report.fn_byte_call_target_witnesses();
                for w in ctw.iter().take(8) {
                    println!("\x20     {w}");
                }
                if ctw.len() > 8 {
                    println!("\x20     … and {} more witnesses", ctw.len() - 8);
                }
            }
            // **The witnesses.** Known answer: none. Every differing function is
            // named with its first disagreeing word, because a count is not
            // something a lane can reproduce from.
            let wit = report.fn_byte_differ_witnesses();
            if !wit.is_empty() {
                let sigs = report.fn_byte_differ_signatures();
                println!(
                    "\x20   DIFFERS WITNESSES — {} distinct functions in {} SIGNATURES \
                     (shape | port/ref/equal words | first disagreeing word):",
                    wit.len(),
                    sigs.len()
                );
                for (sig, n, ex) in sigs.iter().take(40) {
                    println!("\x20     {n:>6}  {sig}   e.g. {ex}");
                }
                if sigs.len() > 40 {
                    println!(
                        "\x20     … and {} more signatures covering {} functions",
                        sigs.len() - 40,
                        sigs.iter().skip(40).map(|(_, n, _)| n).sum::<usize>()
                    );
                }
            }
            // **THE DIFF-SIGNATURE CLUSTER CENSUS** (board #976,
            // `super::fndiff`, `docs/DIFF_STRUCTURE.md`). The witness table
            // above groups by *first wrong word*, which splits one mechanism
            // across as many rows as it has call sites. This groups by the
            // STRUCTURE of the disagreement — how the two bodies align, and
            // which decoded field moved — which is the axis a fix lane can be
            // written against.
            //
            // Printed whether or not `--fnbyte-diff-jsonl` was passed: a census
            // that only some invocations produce is one that goes stale unseen.
            let clusters = report.fndiff_clusters();
            if !clusters.is_empty() {
                let rows = report.emit_total("fndiff-rows");
                let broken = report.emit_total("fndiff-accounting-broken");
                let capped = report.emit_total("fndiff-align-capped");
                println!(
                    "\x20   DIFF STRUCTURE — {rows} signatures in {} clusters \
                     (shape | length | edit shape | field classes).\n\
                     \x20     accounting breaks (equal+sub+del == ref words, known answer 0): \
                     {broken}   LCS-capped rows: {capped}   pure reorderings \
                     (same instruction multiset): {}   first word already wrong: {}",
                    clusters.len(),
                    report.emit_total("fndiff-same-multiset"),
                    report.emit_total("fndiff-first-word"),
                );
                // The alarm the whole census would otherwise be quietly wrong
                // under: a row whose alignment does not add up is a row whose
                // cluster is meaningless, and it must not be readable only as a
                // missing line.
                if broken > 0 {
                    println!(
                        "\x20     DIFF-SIGNATURE ACCOUNTING BROKEN on {broken} rows — the \
                         alignment does not add up and the cluster table above must not be \
                         believed"
                    );
                }
                for (k, n) in clusters.iter().take(25) {
                    println!(
                        "\x20     {n:>6} ({:>5.1}%)  {k}",
                        100.0 * *n as f64 / rows.max(1) as f64
                    );
                }
                if clusters.len() > 25 {
                    println!(
                        "\x20     … and {} more clusters covering {} functions",
                        clusters.len() - 25,
                        clusters.iter().skip(25).map(|(_, n)| n).sum::<usize>()
                    );
                }
                let classes = report.fndiff_classes();
                if !classes.is_empty() {
                    let total: usize = classes.iter().map(|(_, n)| *n).sum();
                    let und = classes
                        .iter()
                        .find(|(k, _)| k == "undecoded")
                        .map(|(_, n)| *n)
                        .unwrap_or(0);
                    println!(
                        "\x20     substituted WORDS by decoded field class ({total} words, \
                         {und} undecoded = {:.1}% — a word is decoded only if its form's \
                         field partition re-encodes it bit-exactly):",
                        100.0 * und as f64 / total.max(1) as f64
                    );
                    let rowstr: Vec<String> =
                        classes.iter().map(|(k, n)| format!("{k} {n}")).collect();
                    println!("\x20       {}", rowstr.join(" · "));
                }
                let firsts = report.fndiff_first_buckets();
                if !firsts.is_empty() {
                    let rowstr: Vec<String> =
                        firsts.iter().map(|(k, n)| format!("w{k}:{n}")).collect();
                    println!("\x20     first divergence, by word index: {}", rowstr.join(" · "));
                }
                println!(
                    "\x20     relocation-aware: {} substitutions and {} deletions sit under a \
                     relocation ({} records not word-aligned, known answer 0)",
                    report.emit_total("fndiff-sub-at-reloc"),
                    report.emit_total("fndiff-del-at-reloc"),
                    report.emit_total("fndiff-reloc-unaligned"),
                );
            }
            // Per-TU FBM, nearest first — the answer to "we are 8/878 exact, how
            // close is the other 870?" stated in TUs rather than in one ratio.
            let by_tu = report.fn_byte_by_tu();
            let buckets = [1.0f64, 0.9, 0.5, 0.1];
            let dist: Vec<String> = buckets
                .iter()
                .map(|b| {
                    let n = by_tu
                        .iter()
                        .filter(|(_, e, d)| *e as f64 / *d as f64 >= *b - 1e-12)
                        .count();
                    format!("≥{:.0}%: {n}", b * 100.0)
                })
                .collect();
            println!(
                "\x20   per-TU FBM over {} TUs with emitted functions — {}",
                by_tu.len(),
                dist.join(", ")
            );
        }
        None => {
            println!(
                "\n\x20 FUNCTION BYTE MATCH: NO-RESULT — no emitted function was graded. \
                 A ratio over zero functions is unrepresentable on purpose (objdiff's \
                 calc_fuzzy_match_percent returns 100.0 here; that is the bug, not the \
                 baseline)."
            );
        }
    }
    // **BLIND REACH (S0) — the same judge again, on the half FBM cannot reach.**
    //
    // Printed in ITS OWN BLOCK under ITS OWN DISCLAIMER, apart from the class
    // table that carries `match`/`mismatch`, because
    // `docs/FUNCTION_BYTE_MATCH.md` §0 is the standing template for every
    // gradient added after FBM and this is one. It is never in `scripts/gate.sh`
    // and it licenses no emit.
    {
        let g = |k: &str| report.emit_total(k);
        let attempted = g("fnbyte-blind-attempted");
        if attempted == 0 {
            // **NO-RESULT, never "0 differs".** A blind block that printed
            // zeros over an empty population would read as "nothing wrong here",
            // which is the exact shape of the twelve-plus absence-as-success
            // defects in this tree's history.
            println!(
                "\n\x20 BLIND REACH (S0): NO-RESULT — the relaxed decode was offered NO \
                 parse-refused function.\n\
                 \x20   Nothing was graded. This is not `blind-differs 0`; a lane quoting a \
                 blind number off this scan has no number."
            );
        } else {
            let level = if g("fnbyte-blind-level|name-from-gl") > 0 {
                "1 (name-from-gl)"
            } else {
                "0 (strict — THE IDENTITY CONTROL; the relaxed decode IS the strict decode here)"
            };
            let (exact, differs, unlow) = (
                g("fnbyte-blind-exact"),
                g("fnbyte-blind-differs"),
                g("fnbyte-blind-unlowerable"),
            );
            let reached = exact + differs;
            let pct = |n: usize| 100.0 * n as f64 / attempted as f64;
            println!(
                "\n\x20 BLIND REACH (S0) — a CHARACTERIZATION instrument, NEVER a gate, and it \
                 LICENSES NO EMIT.\n\
                 \x20   (docs/ROADMAP_SLICING_2026-08-21.md §5 row S0; separation rule \
                 docs/FUNCTION_BYTE_MATCH.md §0.)\n\
                 \x20   FBM grades the functions the reader ACCEPTED. This grades the ones it \
                 REFUSED — `fnbyte-refused-parse`,\n\
                 \x20   the population this harness's own factors doc calls \"the unmeasurable \
                 half\". A candidate body from a\n\
                 \x20   RELAXED DECODE, through the ONE composition, byte-compared against real \
                 c2's own COMDAT bytes.\n\
                 \x20   It answers: is the port's byte-exactness a MODEL, or a FIT? \
                 (§4, the riskiest assumption in the program.)\n\
                 \x20\n\
                 \x20   ** BYTES ONLY. NO RELOCATION VERDICT IS PUBLISHED HERE. ** A relaxed \
                 body may carry a placeholder\n\
                 \x20   symbol, so its relocations would be against a name this instrument \
                 invented. `fnbyte-blind-exact`\n\
                 \x20   is therefore NOT `fnbyte-exact` (which requires byte AND relocation \
                 identity) and the two MUST NOT be summed.\n\
                 \x20\n\
                 \x20   ladder depth: {level}\n\
                 \x20   attempted   {attempted:>8}   THE DENOMINATOR of every line below — the \
                 parse-refused functions offered to the relaxed decode\n\
                 \x20     exact     {exact:>8}  ({:>5.2}%)   relaxed body composed and its bytes \
                 are IDENTICAL to c2's — the catalogue reached past its own admission gate\n\
                 \x20     differs   {differs:>8}  ({:>5.2}%)   complete bytes, and they differ — \
                 A DIRECT PRICE on the wrong emits the next `functions()` widening would ship\n\
                 \x20     unlower.  {unlow:>8}  ({:>5.2}%)   no bytes at all\n\
                 \x20       no-decode   {:>8}   the relaxed decode produced nothing. The bucket \
                 §3's ten constructs (C1-C10) would move; NOT a lowering result\n\
                 \x20       no-select   {:>8}   decoded, and `select_function` declined — the \
                 catalogue has no shape for it\n\
                 \x20       no-compose  {:>8}   selected, and the /Gy composition declined\n\
                 \x20       no-refbytes {:>8}   the REFERENCE COMDAT carried no readable bytes\n\
                 \x20   REACH = {reached} of {attempted} ({:.3}%) — the sub-population the \
                 relaxed decode actually delivered to the lowering.\n\
                 \x20   Read `exact` and `differs` against REACH, never against `attempted`: \
                 outside the reach the lowering was never asked.\n\
                 \x20   differing bodies, word census: port {} · ref {} · EQUAL {} \
                 (equal-words 0 over a nonzero differs means not one word agreed anywhere)\n\
                 \x20   CONTROLS, each a DEFECT count with a known answer of 0 — partition \
                 {} · population {} · census-desync {}\n\
                 \x20     population compares `attempted` against the sibling FBM walk's \
                 `fnbyte-refused-parse` ({}), in the same iteration that filed it.",
                pct(exact),
                pct(differs),
                pct(unlow),
                g("fnbyte-blind-unlowerable|no-decode"),
                g("fnbyte-blind-unlowerable|no-select"),
                g("fnbyte-blind-unlowerable|no-compose"),
                g("fnbyte-blind-unlowerable|no-refbytes"),
                100.0 * reached as f64 / attempted as f64,
                g("fnbyte-blind-differs-port-words"),
                g("fnbyte-blind-differs-ref-words"),
                g("fnbyte-blind-differs-equal-words"),
                g("fnbyte-blind-partition-broken"),
                g("fnbyte-blind-population-broken"),
                g("fnbyte-blind-census-desync"),
                g("fnbyte-refused-parse"),
            );
            // **WHICH REFUSAL CLASSES THE REACHED SUB-POPULATION CONTAINS — and
            // which it does not.** Without this the block above is exactly the
            // artefact `ranking instruments measure themselves` warns about:
            // "the lowering generalises" is unreadable unless you know which
            // gate it generalised past, and a frozen holdout is only as good as
            // the classes it happens to contain. Printed for BOTH sides, because
            // the two-sided price is per-class or it is not actionable.
            for (label, prefix) in [
                ("exact", "fnbyte-blind-exact|"),
                ("differs", "fnbyte-blind-differs|"),
            ] {
                let mut by_class: std::collections::BTreeMap<&str, usize> = Default::default();
                for r in &report.results {
                    for (k, n) in &r.emit {
                        if let Some(cls) = k.strip_prefix(prefix) {
                            *by_class.entry(cls).or_default() += n;
                        }
                    }
                }
                let mut rows: Vec<_> = by_class.into_iter().collect();
                rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
                let total: usize = rows.iter().map(|(_, n)| n).sum();
                println!(
                    "\x20   blind-{label} by the STRICT reader's refusal class \
                     ({} distinct, {total} functions) — the reach is only as good as the \
                     classes it contains:",
                    rows.len()
                );
                if rows.is_empty() {
                    println!("\x20     (none)");
                }
                for (cls, n) in rows.iter().take(12) {
                    println!("\x20     {n:>6}  {cls}");
                }
                if rows.len() > 12 {
                    println!("\x20     … and {} more distinct classes", rows.len() - 12);
                }
            }
        }
    }
    render_decode_reach(report);
    render_symbind(report);
    // ---- W-FENCECOUNT: the per-fence hold-out counter -----------------------
    //
    // The instrument the two-sided fence-pricing rule (CLAUDE.md) needs on the
    // scan itself: for each decode-gate cause, the TUs it holds ALONE, the
    // subset whose every emitted body is already byte-exact (the shape of
    // vsnprnc.cpp before w-fence2 paid its fence, board #2470), and the TUs it
    // merely blocks FIRST. Diagnostic only; nothing branches on it.
    {
        let fence = report.fence_blocks();
        println!(
            "\n\x20 FENCE-BLOCKS-EXACT (w-fencecount) — TUs held out of `match` per decode-gate \
             fence. `sole` = the TU's ONLY firing cause; `exact` = sole AND every emitted body \
             FnByte-exact (`bodies` counts them); `first` = first blocker of a multi-cause TU. \
             TWO CAVEATS, standing: a first-blocker count is NOT a distance (the port stops at \
             its first refusal, so every held TU names one blocker however many it has); and \
             the `locally-defined-callee` row is `decode_causes`' BROAD re-ask of the inline \
             fence, which can fire where the narrowed gate exempts (see \
             `GapReport::fence_blocks`). Machine keys: `gap-metric fence-*`, all causes, zeros \
             included."
        );
        let firing: Vec<(&String, &super::FenceCauseRow)> = fence
            .per_cause
            .iter()
            .filter(|(_, c)| c.sole + c.exact_tus + c.first_of_multi > 0)
            .collect();
        if firing.is_empty() {
            // The zero is stated positively, over its population — never as an
            // absent block (trap 5).
            println!(
                "\x20   no fence fires on this scan: {} held TU(s), every row zero",
                fence.held_tus
            );
        }
        for (cause, row) in firing {
            println!(
                "\x20   {cause:<34} sole {:>4}  exact {:>4}  bodies {:>5}  first-of-multi {:>4}",
                row.sole, row.exact_tus, row.exact_bodies, row.first_of_multi
            );
        }
        let attributed: usize = fence
            .per_cause
            .values()
            .map(|c| c.sole + c.first_of_multi)
            .sum();
        println!(
            "\x20   controls: held {} = attributed {} + arity-broken {} (accounting-broken {}); \
             cause-firings {} over {} held TUs (arity); residue-no-cause {} (known 0); \
             decodes-not-match {} (codegen-gap/mismatch/port-error — outside the fence family); \
             class-disagree {} (known 0); match TUs checked {}, {} carrying a cause (known 0)",
            fence.held_tus,
            attributed,
            fence.arity_broken,
            if fence.held_tus != attributed + fence.arity_broken { 1 } else { 0 },
            fence.cause_firings,
            fence.held_tus,
            fence.residue_no_cause,
            fence.decodes_not_match,
            fence.class_disagree,
            fence.match_tus_checked,
            fence.on_match_tu,
        );
    }

    render_plan(report);

    println!(
        "\n\x20 GAP-METRICS — stable `key value` pairs for scripts/status.sh; keys are an \
         interface, do not rename. The projection `emit-predicate-worth` = \
         `b-and-c` − `a-and-b-and-c` is derived HERE on purpose (board #213):"
    );
    for (k, v) in report.metrics() {
        println!("\x20   gap-metric {k} {v}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tu(keys: &[(&str, usize)]) -> TuResult {
        let mut r = TuResult {
            src: "t.cpp".into(),
            class: TuClass::CodegenGap,
            reason: String::new(),
            detail: String::new(),
            ex_len: 0,
            fn_names: 0,
            replay_ok: None,
            fn_total: 0,
            fn_in_class: 0,
            fn_blockers: Default::default(),
            fn_frames: Default::default(),
            fn_cflow: Default::default(),
            fn_cflow_off: Default::default(),
        fn_cfg_admit: Default::default(),
        fn_decode: Default::default(),
        fn_symbind: Default::default(),
            fn_eh: Default::default(),
            fn_dispatch: Default::default(),
            fn_complete: Default::default(),
            fn_prod: Default::default(),
            fn_gate_refusals: Default::default(),
            gate_cause: None,
            gate_causes: Vec::new(),
            gl_body_starts: None,
            selective_bind: None,
            bind_checks: Default::default(),
            emit: Default::default(),
            emit_blockers: Default::default(),
            emit_witness: Vec::new(),
        fndiff: Vec::new(),
            plan: Default::default(),
        };
        for (k, v) in keys {
            r.emit.insert((*k).into(), *v);
        }
        r
    }

    /// **The three states, and that they are three** (board **#742**).
    ///
    /// The whole point of the column is the difference between *"this obj has no
    /// `$M`"* and *"we could not read this obj"*. Collapsing them would report an
    /// undecodable obj as label-free, which is the flattering direction and the
    /// one `docs/STATUS.md` trap 5 records twelve times.
    #[test]
    fn the_label_channel_distinguishes_label_free_from_unreadable_from_not_evaluated() {
        assert_eq!(
            label_channel(&tu(&[("emit-label-readable", 1)])),
            "label-free",
            "read, and it holds no labels"
        );
        assert_eq!(
            label_channel(&tu(&[("emit-label-readable", 1), ("emit-label-syms", 3)])),
            "labels 3"
        );
        assert_eq!(
            label_channel(&tu(&[("emit-label-unreadable", 1)])),
            "label ??",
            "an obj that did not decode is NOT label-free"
        );
        assert_eq!(
            label_channel(&tu(&[])),
            "label n/e",
            "no key at all means the obj was never read — a third answer, not a pass"
        );
    }

    /// An `emit-label-syms` of 0 recorded *beside* the readable flag is
    /// `label-free`, and an unreadable obj that somehow also carried a count is
    /// still `label ??`. The unreadable arm is checked FIRST for that reason and
    /// the ordering is pinned here rather than left to the reader.
    #[test]
    fn unreadable_wins_over_any_count_that_was_also_recorded() {
        assert_eq!(
            label_channel(&tu(&[
                ("emit-label-unreadable", 1),
                ("emit-label-readable", 1),
                ("emit-label-syms", 3),
            ])),
            "label ??"
        );
    }
}
