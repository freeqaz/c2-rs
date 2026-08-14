//! The scan's own printed report of the factor model. Split out of `gap.rs`
//! unchanged; see [`super`] for the module docs.

use super::fnbytes::byte_fraction_exact;
use super::factors::CfgReach;
use super::{GapReport, TuClass, TuResult, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

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
         observed: `IlFunction::label_slots` returns `None` for four of the five loop shapes the \
         port emits, so a TU pairing one of THOSE with a framed function still refuses. **The \
         fifth is lifted** (lane `w-fenceb`, board #746's fence B): the pointer-walk loop's \
         charge was measured at 2 against `fixtures/cpp/whash_loop_then_framed.cpp`'s own obj \
         and that TU is now `match` at `/O1` — the first time this counter was PAID rather than \
         refused (boards #746/#747/#3091, and its control `whash_ptr_walk_loop.cpp`). \
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
