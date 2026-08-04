#!/bin/sh
# THE MODE-LANE GATE — run every lane in `scripts/lanes.txt`, one result each.
#
# ---- why this exists -----------------------------------------------------------
#
# `mode_lane.sh` runs ONE lane and has always worked. Nothing enumerated the lanes,
# so the set that actually ran on any given day was the set somebody remembered to
# type — and the four recorded through `docs/` (`/Ox`, `/O1`, `/O2`, `/Ox /Gy`)
# contain no `/EH` at all, on a workload that compiles `/EHsc` on every TU. Two
# `/EHsc` lanes were added, went green, and caught a live wrong-bytes emit every
# other lane was blind to; nothing whatsoever made them run again.
#
# A lane that exists but is not enumerated is a lane that does not run. The list is
# now data (`scripts/lanes.txt`) and this is the one command that runs it.
#
# ---- what this gate promises ---------------------------------------------------
#
# The promise is deliberately stated as a POSITIVE: **every lane in the registry
# produced a result, and the gate says how many.** Not "no lane failed" — the
# expensive failure class on this project is not a lane going red, it is a lane
# going ABSENT and the absence reading as zero. Eight instruments here have now
# reported green from an absence, including one whose every check `sed`-ed a number
# out of a report and read the missing number as 0, passing a run that graded
# literally nothing.
#
# So, concretely:
#
#   * Each lane must print a `LANE-RESULT` line. The gate parses that line and
#     re-derives the verdict from its fields. **A zero exit status is not accepted
#     as evidence that a lane ran** — a lane that dies, is killed, or is skipped by
#     the loop prints no such line and is reported `NO-RESULT`, which fails.
#   * `PASS` additionally requires `graded > 0`: a lane that submitted 197 fixtures
#     and graded none of them is not a passing lane, whatever its exit status.
#     (That is the toolchain PRESENT and every capture failing — a relative outdir,
#     an exhausted tmpfs inode table, a bad flag — and it is a different thing from
#     the toolchain being absent.)
#   * The result table is rendered by walking the REGISTRY, not by walking whatever
#     result files happen to exist, and the number of rendered rows is compared
#     against the registry length before any verdict is printed. A lane cannot
#     vanish out of the table.
#   * `SKIP` is its own verdict and prints as `SKIP`, never as `PASS`. An all-SKIP
#     run prints `GATE: SKIPPED` and says in the headline that nothing was graded.
#     It exits 0 because CLAUDE.md requires the toolchain-absent path to degrade
#     cleanly — but it cannot be mistaken for green by anything reading the output.
#   * A PARTIAL skip fails. If the toolchain is present, every lane runs; some
#     lanes skipping and others not means a lane declined for a lane-specific
#     reason, which is a fault, not a degradation.
#
# `--selftest` proves all of the above against fabricated lane logs, needs no
# toolchain, and is the answer to "has anyone ever seen this gate fail?".
#
# ---- the GENERATED SWEEP is part of this gate (2026-08-04) ----------------------
#
# `scripts/expr_sweep.sh` enumerates ~14.5k small TUs and grades every one against
# the real `c2.dll`. Until today this file had **zero** references to it, and that
# blindness is not theoretical: board **#232** was a live `Port=Mismatch` — a
# refusal that had become a wrong emit, the one direction the correctness rule
# exists to forbid — and it survived **255 commits** (`d0d8a98..be86f9d`, two
# days) while every lane's gate run and every coordinator re-gate came back
# green. Count it yourself before requoting it: the `376` on #232's row is the
# BISECT RANGE from the last recorded green sweep, which starts before the defect
# existed, and it is a different quantity from how long the defect survived. The 12 mode lanes grade hand-built
# fixtures and `c2rs gap` grades workload TUs; **neither generates that shape.** A
# check that runs when somebody remembers it is a check that does not run — the
# same defect `lanes.txt` was written for, one level out.
#
# So the sweep is now a ROW in the table above, produced by the same
# walk-and-count discipline as a lane and subject to the same rules:
#
#   * It runs UNCONDITIONALLY. There is no `--no-sweep`; an omittable check is an
#     omitted check, and this project has twelve recorded instances of an absence
#     reading as a success.
#   * Its POSITIVE check is re-derived here, not believed. The sweep prints
#     `sweeping R of T generated cases` (and `sweep_gen.py` has already reconciled
#     T against the `.cpp` on disk) and then `checked=C mismatches=M`. **This gate
#     fails unless `C == R`** — a run that graded fewer cases than it selected is a
#     dead worker, not a pass, and `M == 0` over `C == 0` is the vacuous green the
#     whole file exists to forbid.
#   * `--sweep-cases N` is the only way to grade less than the whole corpus, and a
#     sampled run **cannot print an unqualified PASS** — the verdict reads
#     `GATE: PASS (SWEEP SAMPLED)` and says what it did not establish, exactly as
#     `GATE: SKIPPED` does. The sample is a STRIDE across the sorted case list, not
#     a prefix. Measured: a 400-case budget reaches **1 of 47 fragments** as a
#     prefix and **46 of 47** as a stride, and #232's case is **line 9,538 of
#     14,484** — a prefix cheap enough to want was blind to it by construction.
#   * The sweep grades the SAME PINNED BINARY as the lanes (`C2RS_BIN`), so the
#     table is one run of one binary rather than two runs that might not be.
#   * `C2RS_SWEEP_ONLY` is unset for the gate's run. It filters fragments and makes
#     the total meaningless by design; a gate over a filtered corpus is not a gate.
#
# COST, measured on this box 2026-08-04 (32 cores, warm capture cache), and it is
# the whole basis of the "unconditional" decision:
#
#     12 lanes alone, --jobs 8                      7 s
#     sweep alone, serial (as it was written)   9 min 51 s
#     sweep alone, --jobs 8                     1 min 26 s
#     THIS GATE, --jobs 8                       1 min 34 s
#
# Both sweep runs printed `checked=14484 mismatches=0` — the parallel split is an
# equivalence, not an approximation. Every case is an independent `c2rs diff` and
# the loop was serial for no reason. **Parallelising it is what makes
# "unconditional" affordable: the trade-off was resolved by removing the cost, not
# by making the check optional.**
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/gate.sh                       run every lane in the registry + the sweep
#   scripts/gate.sh --lane O1-Oi-EHsc     run named lanes only (repeatable)
#   scripts/gate.sh --jobs 4              lanes in parallel (default 4); also the
#                                         sweep's grading concurrency
#   scripts/gate.sh --sweep-cases 400     STRIDED subset — never an unqualified PASS
#   scripts/gate.sh --cross-cells 4000    ditto, for the mode-cross row
#   scripts/gate.sh --list                print the registry and exit
#   scripts/gate.sh --check               validate the registry only; no toolchain
#   scripts/gate.sh --selftest            prove the gate fails when it should
#   scripts/gate.sh --work DIR            run directory (default /tmp/c2rs-gate-$$)
#
# Lane run directories stay PER LANE, inherited from `mode_lane.sh`, which uses one
# per mode precisely because a shared directory had concurrent lanes overwriting
# each other's flags file and report and the mismatch count then came out of
# whichever report won.
set -eu

TAB=$(printf '\t')
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
registry="${C2RS_LANES:-$repo_root/scripts/lanes.txt}"
jobs=4
work=""
want=""
mode=run
sweep_cases=0
cross_cells=0

while [ $# -gt 0 ]; do
    case "$1" in
        --list)     mode=list ;;
        --check)    mode=check ;;
        --selftest) mode=selftest ;;
        --lane)     shift; want="$want $1" ;;
        --jobs)     shift; jobs="$1" ;;
        --sweep-cases) shift; sweep_cases="$1" ;;
        --cross-cells) shift; cross_cells="$1" ;;
        --work)     shift; work="$1" ;;
        --registry) shift; registry="$1" ;;
        -h|--help)  sed -n '2,/^set -eu$/p' "$0" | sed '$d'; exit 0 ;;
        *) echo "gate.sh: unknown argument '$1' (try --help)" >&2; exit 2 ;;
    esac
    shift
done
[ -n "$work" ] || work="/tmp/c2rs-gate-$$"

# --------------------------------------------------------------------------------
# The registry, parsed once. `slug<TAB>flags`, one per line, comments stripped.
# --------------------------------------------------------------------------------
parse_registry() {
    _pr_src="$1"; _pr_dst="$2"
    if [ ! -f "$_pr_src" ]; then
        echo "FATAL: no lane registry at $_pr_src" >&2
        return 1
    fi
    # Every line with content must BECOME a lane. A row that carries a slug and no
    # flags used to be dropped by the `NF >= 2` filter, so `--list` reported one
    # lane fewer than the file contains and nothing said so — a registry silently
    # short a row is the same absence-reads-as-success bug one layer further out
    # than the one this gate was built for. Malformed rows are named and fatal.
    sed 's/#.*//' "$_pr_src" | awk 'NF > 0' > "$_pr_dst.rows"
    awk 'NF >= 2 { slug=$1; $1=""; sub(/^[ \t]+/,""); printf "%s\t%s\n", slug, $0 }' \
        "$_pr_dst.rows" > "$_pr_dst"
    _pr_rows=$(wc -l < "$_pr_dst.rows")
    _pr_n=$(wc -l < "$_pr_dst")
    if [ "$_pr_n" -ne "$_pr_rows" ]; then
        echo "FATAL: $_pr_src has $_pr_rows non-comment rows but only $_pr_n parse as lanes." >&2
        echo "  A row needs a slug AND at least one flag. Offending row(s):" >&2
        awk 'NF > 0 && NF < 2 { printf "    %s\n", $0 }' "$_pr_dst.rows" >&2
        return 1
    fi
    # An EMPTY registry is a gate that runs nothing and exits 0 — the exact shape
    # this whole file exists to make impossible. It is a hard error, never a pass.
    if [ "$_pr_n" -eq 0 ]; then
        echo "FATAL: lane registry $_pr_src defines NO lanes." >&2
        echo "  A gate with an empty lane list grades nothing and would exit 0." >&2
        return 1
    fi
    _pr_dup=$(cut -f1 "$_pr_dst" | sort | uniq -d)
    if [ -n "$_pr_dup" ]; then
        echo "FATAL: duplicate lane slug(s) in $_pr_src: $_pr_dup" >&2
        echo "  Two rows under one slug means one silently replaces the other's" >&2
        echo "  result while the table still shows the expected number of rows." >&2
        return 1
    fi
    return 0
}

# --------------------------------------------------------------------------------
# Verdict for ONE lane, derived from its log. Deliberately a pure function of the
# log text, so `--selftest` can drive it with fabricated logs and no toolchain.
#
# Emits: <verdict>|<graded>|<total>|<match>|<mismatch>|<detail>
# --------------------------------------------------------------------------------
lane_verdict() {
    _lv_log="$1"; _lv_status="${2:-}"

    if [ ! -f "$_lv_log" ]; then
        echo "NO-RESULT|0|0|0|0|the lane produced no log at all"
        return 0
    fi
    _lv_line=$(grep -m1 '^LANE-RESULT ' "$_lv_log" 2>/dev/null || true)
    if [ -z "$_lv_line" ]; then
        echo "NO-RESULT|0|0|0|0|log has no LANE-RESULT line (exit ${_lv_status:-?})"
        return 0
    fi

    _lv_v=$(printf '%s\n' "$_lv_line" | awk '{print $2}')
    _lv_g=$(printf '%s\n' "$_lv_line" | sed -n 's/.* graded=\([0-9][0-9]*\).*/\1/p')
    _lv_t=$(printf '%s\n' "$_lv_line" | sed -n 's/.* total=\([0-9][0-9]*\).*/\1/p')
    _lv_m=$(printf '%s\n' "$_lv_line" | sed -n 's/.* match=\([0-9][0-9]*\).*/\1/p')
    _lv_x=$(printf '%s\n' "$_lv_line" | sed -n 's/.* mismatch=\([0-9][0-9]*\).*/\1/p')

    # Every field must be PRESENT. An unparseable result line is a lane that did
    # not report — NOT a lane that reported zeros. That distinction is the entire
    # bug class this gate is built around.
    if [ -z "$_lv_g" ] || [ -z "$_lv_t" ] || [ -z "$_lv_m" ] || [ -z "$_lv_x" ]; then
        echo "NO-RESULT|0|0|0|0|malformed LANE-RESULT line"
        return 0
    fi

    case "$_lv_v" in
    SKIP)
        echo "SKIP|0|$_lv_t|0|0|toolchain absent"
        ;;
    PASS)
        # Re-derive rather than believe. A lane claiming PASS while having graded
        # nothing, or while carrying a mismatch, is a lane wrong about itself, and
        # the gate is the second opinion.
        if [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|0|$_lv_t|$_lv_m|$_lv_x|claimed PASS having graded 0 of $_lv_t"
        elif [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS with mismatch=$_lv_x"
        elif [ "${_lv_status:-0}" != "0" ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS but exited $_lv_status"
        else
            echo "PASS|$_lv_g|$_lv_t|$_lv_m|$_lv_x|"
        fi
        ;;
    FAIL)
        if [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|MISMATCH — the port emitted wrong bytes"
        elif [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|vacuous — 0 of $_lv_t graded"
        else
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|lane reported FAIL"
        fi
        ;;
    *)
        echo "NO-RESULT|0|0|0|0|unrecognized verdict '$_lv_v'"
        ;;
    esac
    return 0
}

# --------------------------------------------------------------------------------
# Verdict for the GENERATED SWEEP, derived from its log. Same contract as
# `lane_verdict`: a pure function of the log text, so `--selftest` drives it with
# fabricated logs and no toolchain.
#
# The sweep's own positive check is RE-DERIVED here rather than believed. It prints
#
#     sweeping 14484 of 14484 generated cases      <- selected R of generated T
#     checked=14484 mismatches=0                   <- graded C, found M
#
# and `sweep_gen.py` has already failed the run if T disagreed with the `.cpp` on
# disk. This function's job is the next link: **C must equal R.** A sweep whose
# workers died grades fewer cases than it selected and still prints
# `mismatches=0`, which is the exact shape of a green from an absence.
#
# Emits: <verdict>|<checked>|<selected>|<total>|<mismatch>|<detail>
# --------------------------------------------------------------------------------
sweep_verdict() {
    _sv_log="$1"; _sv_status="${2:-}"

    if [ ! -f "$_sv_log" ]; then
        echo "NO-RESULT|0|0|0|0|the instrument produced no log at all"
        return 0
    fi
    # SKIP is the toolchain-absent path and is the ONLY path allowed to have no
    # counts. It is checked first so a skipped run is never read as malformed.
    if grep -q '^SKIP: toolchain absent' "$_sv_log" 2>/dev/null; then
        echo "SKIP|0|0|0|0|toolchain absent"
        return 0
    fi

    # The unit word is deliberately NOT anchored: `expr_sweep.sh` says
    # "generated cases" and `mode_cross.sh` says "case-lane cells", and both are
    # ruled on by this one function. A second copy of these rules for the second
    # instrument is the "one rule, two implementations" shape `docs/GAPS.md` §6
    # keeps recording, and it is how the two would drift into disagreeing about
    # what a short count means.
    _sv_sel=$(sed -n 's/^sweeping \([0-9][0-9]*\) of \([0-9][0-9]*\) .*/\1/p' "$_sv_log" | head -1)
    _sv_tot=$(sed -n 's/^sweeping \([0-9][0-9]*\) of \([0-9][0-9]*\) .*/\2/p' "$_sv_log" | head -1)
    _sv_c=$(sed -n 's/^checked=\([0-9][0-9]*\) mismatches=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_m=$(sed -n 's/^checked=\([0-9][0-9]*\) mismatches=\([0-9][0-9]*\).*/\2/p' "$_sv_log" | head -1)
    # `checked` is only "cases the loop reached". `graded` is "cases the ORACLE
    # ruled on", and until 2026-08-04 the two were reported as one number while
    # 96 of 14,635 cases had no reference obj at all — `c2rs diff` prints four
    # verdicts and the sweep's classifier recognized one of them. Both fields are
    # now required; a log carrying only the old line is NO-RESULT, not a log
    # reporting `ungraded=0`.
    _sv_g=$(sed -n 's/^checked=.* graded=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_u=$(sed -n 's/^checked=.* ungraded=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_k=$(sed -n 's/^checked=.* unknown=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)

    # Both lines must be PRESENT. A missing `sweeping` line is a run whose corpus
    # never got enumerated; a missing `checked=` line is a run that died. Neither
    # is a run that found nothing.
    if [ -z "$_sv_sel" ] || [ -z "$_sv_tot" ]; then
        echo "NO-RESULT|0|0|0|0|log has no 'sweeping R of T' line (exit ${_sv_status:-?})"
        return 0
    fi
    if [ -z "$_sv_c" ] || [ -z "$_sv_m" ]; then
        echo "NO-RESULT|0|$_sv_sel|$_sv_tot|0|log has no 'checked=' line (exit ${_sv_status:-?})"
        return 0
    fi
    if [ -z "$_sv_g" ] || [ -z "$_sv_u" ] || [ -z "$_sv_k" ]; then
        echo "NO-RESULT|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|count line has no graded=/ungraded=/unknown= — a pre-2026-08-04 sweep, whose 'checked' includes cases the oracle never ruled on"
        return 0
    fi

    if [ "$_sv_c" -ne "$_sv_sel" ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|SHORT — selected $_sv_sel cases, reached $_sv_c|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_c" -eq 0 ]; then
        echo "FAIL|0|$_sv_sel|$_sv_tot|$_sv_m|vacuous — 0 cases reached|0|0"
        return 0
    fi
    if [ "$_sv_k" -ne 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|UNRECOGNIZED verdict on $_sv_k case(s) — an unenumerated verdict is the next silence|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_g" -eq 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|vacuous — $_sv_c cases reached and NONE graded|0|$_sv_u"
        return 0
    fi
    if [ "$_sv_m" -ne 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|MISMATCH — the port emitted wrong bytes on $_sv_m case(s)|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_sel" -lt "$_sv_tot" ]; then
        echo "SAMPLED|$_sv_c|$_sv_sel|$_sv_tot|0|a STRIDED sample, not the corpus|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "${_sv_status:-0}" != "0" ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|graded every case cleanly but exited $_sv_status|$_sv_g|$_sv_u"
        return 0
    fi
    echo "PASS|$_sv_c|$_sv_sel|$_sv_tot|0||$_sv_g|$_sv_u"
    return 0
}

# --------------------------------------------------------------------------------
# Walk the REGISTRY (never the directory listing) and produce one row per lane.
# --------------------------------------------------------------------------------
collect() {
    _c_reg="$1"; _c_run="$2"; _c_out="$3"
    : > "$_c_out"
    while IFS="$TAB" read -r _c_slug _c_flags; do
        [ -n "$_c_slug" ] || continue
        _c_st=""
        if [ -f "$_c_run/$_c_slug.status" ]; then _c_st=$(cat "$_c_run/$_c_slug.status"); fi
        _c_v=$(lane_verdict "$_c_run/$_c_slug.log" "$_c_st")
        printf '%s\t%s\t%s\n' "$_c_slug" "$_c_flags" "$_c_v" >> "$_c_out"
    done < "$_c_reg"
}

# --------------------------------------------------------------------------------
# THE GENERATED-INSTRUMENT ROWS. Two of them now, and they obey one set of rules.
#
#   expr-sweep   14,635 generated cases at ONE profile (`c2rs diff` hardcodes
#                `/Ox /GS- /c`).
#   mode-cross   the PRODUCT of those cases with `scripts/lanes.txt`, minus the
#                cells `scripts/mode_invariance.py` proved redundant.
#
# Both are ruled on by `sweep_verdict` and both are rendered and checked by the
# loops below, because a second copy of "what does a short count mean" is how the
# two would come to disagree about it.
# --------------------------------------------------------------------------------
gen_tuple() {   # <SWEEP|CROSS>
    case "$1" in
        SWEEP) printf '%s\n' "$_d_sw" ;;
        CROSS) printf '%s\n' "$_d_cx" ;;
    esac
}
gen_name() {
    case "$1" in
        SWEEP) echo "expr-sweep" ;;
        CROSS) echo "mode-cross" ;;
    esac
}
gen_unit() {
    case "$1" in
        SWEEP) echo "generated cases" ;;
        CROSS) echo "case-lane cells" ;;
    esac
}
gen_why() {
    case "$1" in
        SWEEP) echo "board #232: the 12 mode lanes grade hand-built fixtures and
  \`c2rs gap\` grades workload TUs; neither GENERATES the shapes it covers." ;;
        CROSS) echo "w-order Y-a: the sweep runs ONE profile, so a defect that needs
  \`/EHsc\` as well as its shape is invisible to every instrument without this row." ;;
    esac
}

decide() {
    _d_reg="$1"; _d_res="$2"; _d_run="${3:-}"; _d_sw="${4:-}"; _d_filt="${5:-}"
    _d_cx="${6:-}"
    _d_n=$(wc -l < "$_d_reg")
    _d_rows=$(wc -l < "$_d_res")

    echo
    echo "LANE                 VERDICT     graded/total  match  mismatch  flags"
    echo "-------------------- ---------- ------------- ------ --------- --------------------"
    awk -F"$TAB" '{
        split($3, f, "|")
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n", $1, f[1], f[2], f[3], f[4], f[5],
               $2, (f[6] == "" ? "" : "   <- " f[6])
    }' "$_d_res"

    # The generated instruments are ROWS of this table, not notes beside it. **An
    # ABSENT verdict is a failure**, checked before anything else about the lanes:
    # a gate that forgot to run one of them is the state this addition exists to
    # make impossible, and it must not be able to reach a PASS line below.
    for _g in SWEEP CROSS; do
        if [ -z "$(gen_tuple "$_g")" ]; then
            echo
            echo "GATE: FAIL — no $(gen_name "$_g") verdict was produced at all."
            echo "  \`scripts/$( [ "$_g" = SWEEP ] && echo expr_sweep.sh || echo mode_cross.sh )\` is part of this gate."
            echo "  $(gen_why "$_g")"
            return 1
        fi
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n" \
            "$(gen_name "$_g")" \
            "$(gen_tuple "$_g" | cut -d'|' -f1)" \
            "$(gen_tuple "$_g" | cut -d'|' -f2)" \
            "$(gen_tuple "$_g" | cut -d'|' -f3)" \
            "$(gen_tuple "$_g" | cut -d'|' -f7)" \
            "$(gen_tuple "$_g" | cut -d'|' -f5)" \
            "$(gen_unit "$_g") (of $(gen_tuple "$_g" | cut -d'|' -f4))" \
            "$([ -z "$(gen_tuple "$_g" | cut -d'|' -f6)" ] && echo "" || echo "   <- $(gen_tuple "$_g" | cut -d'|' -f6)")"
    done
    echo

    _d_swv=$(printf '%s\n' "$_d_sw" | cut -d'|' -f1)
    _d_swc=$(printf '%s\n' "$_d_sw" | cut -d'|' -f2)
    _d_swsel=$(printf '%s\n' "$_d_sw" | cut -d'|' -f3)
    _d_swtot=$(printf '%s\n' "$_d_sw" | cut -d'|' -f4)
    _d_swm=$(printf '%s\n' "$_d_sw" | cut -d'|' -f5)
    _d_swg=$(printf '%s\n' "$_d_sw" | cut -d'|' -f7)
    _d_swu=$(printf '%s\n' "$_d_sw" | cut -d'|' -f8)
    _d_cxv=$(printf '%s\n' "$_d_cx" | cut -d'|' -f1)
    _d_cxg=$(printf '%s\n' "$_d_cx" | cut -d'|' -f7)
    _d_cxsel=$(printf '%s\n' "$_d_cx" | cut -d'|' -f3)
    _d_cxtot=$(printf '%s\n' "$_d_cx" | cut -d'|' -f4)

    # COMPLETENESS FIRST, and as its own statement. If the table has fewer rows
    # than the registry has lanes, nothing else printed here means anything — a
    # lane silently dropped from the walk is precisely how an absence becomes a
    # green. Checked before any verdict is computed.
    if [ "$_d_rows" -ne "$_d_n" ]; then
        echo "GATE: FAIL — the registry has $_d_n lanes and the table has $_d_rows rows."
        echo "  Rows are produced by walking the registry, so this means the walk itself"
        echo "  broke. No verdict below this line would be trustworthy."
        return 1
    fi

    _d_pass=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="PASS") c++} END{print c+0}' "$_d_res")
    _d_fail=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") c++} END{print c+0}' "$_d_res")
    _d_skip=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") c++} END{print c+0}' "$_d_res")
    _d_none=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") c++} END{print c+0}' "$_d_res")
    _d_graded=$(awk -F"$TAB" '{split($3,f,"|"); g+=f[2]} END{print g+0}' "$_d_res")

    echo "lanes:  $_d_n in the registry — $_d_pass PASS, $_d_fail FAIL, $_d_skip SKIP, $_d_none NO-RESULT"
    echo "graded: $_d_graded fixture-verdicts across all lanes"
    echo "sweep:  $_d_swv — $_d_swc of $_d_swsel selected cases reached, ${_d_swg:-?} GRADED by the"
    echo "        oracle (${_d_swu:-?} ungraded: no reference obj), $_d_swm mismatch (corpus $_d_swtot)"
    echo "cross:  $_d_cxv — ${_d_cxg:-?} of $_d_cxsel selected cells graded, $(printf '%s\n' "$_d_cx" | cut -d'|' -f5) mismatch (product $_d_cxtot)"
    if [ -n "$_d_run" ] && [ -d "$_d_run" ]; then echo "logs:   $_d_run/<lane>.log, $_d_run/sweep.log, $_d_run/cross.log"; fi

    # The generated instruments' failures are ruled on FIRST when they carry a
    # mismatch, because a mismatch outranks every other piece of work (CLAUDE.md)
    # and burying it under a lane table is how it goes unread.
    for _g in SWEEP CROSS; do
        _gv=$(gen_tuple "$_g" | cut -d'|' -f1)
        _gm=$(gen_tuple "$_g" | cut -d'|' -f5)
        _gd=$(gen_tuple "$_g" | cut -d'|' -f6)
        _gt=$(gen_tuple "$_g" | cut -d'|' -f4)
        if [ "$_gv" = "FAIL" ]; then
            echo
            echo "GATE: FAIL — $(gen_name "$_g") failed: $_gd"
            if [ "${_gm:-0}" -gt 0 ]; then
                echo
                echo "  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***"
                echo "  The real c2.dll under wibo plus a byte-exact obj compare is the sole"
                echo "  judge; outside its class the port must REFUSE, not mis-emit."
                _gl="$_d_run/$( [ "$_g" = SWEEP ] && echo sweep.log || echo cross.log )"
                if [ -n "$_d_run" ] && [ -f "$_gl" ]; then
                    grep '^MISMATCH ' "$_gl" | sed 's/^/    /'
                fi
            fi
            return 1
        fi
        if [ "$_gv" = "NO-RESULT" ]; then
            echo
            echo "GATE: FAIL — $(gen_name "$_g") produced NO RESULT: $_gd"
            echo "  An instrument that did not run is a failure, not a pass. Nothing in this"
            echo "  run establishes anything about the ~$_gt $(gen_unit "$_g") it covers."
            return 1
        fi
    done

    if [ "$_d_none" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_none lane(s) produced NO RESULT:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        echo "  A lane that did not run is a failure, not a pass. Nothing in this run"
        echo "  establishes anything about the configurations those lanes cover."
        return 1
    fi

    if [ "$_d_fail" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_fail lane(s) failed:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        if awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL" && f[5]+0 > 0) found=1} END{exit !found}' "$_d_res"; then
            echo
            echo "  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***"
            echo "  The real c2.dll under wibo plus a byte-exact obj compare is the sole"
            echo "  judge; outside its class the port must REFUSE, not mis-emit."
        fi
        return 1
    fi

    if [ "$_d_skip" -eq "$_d_n" ]; then
        # Toolchain absence must skip the generated instruments too. If it did
        # not, one of them resolved a toolchain the others could not, which is a
        # fault in the resolution, not a degradation — the partial-skip rule
        # applied across the gate's halves instead of across the lanes.
        for _g in SWEEP CROSS; do
            _gv=$(gen_tuple "$_g" | cut -d'|' -f1)
            if [ "$_gv" != "SKIP" ]; then
                echo
                echo "GATE: FAIL — all $_d_n lanes skipped but $(gen_name "$_g") reported $_gv."
                echo "  Toolchain absence skips EVERYTHING. One part resolving a toolchain"
                echo "  another could not is a fault in the resolution."
                return 1
            fi
        done
        echo
        echo "GATE: SKIPPED — all $_d_n lanes, the sweep and the cross skipped, NOTHING WAS GRADED."
        echo "  The toolchain is absent (see CLAUDE.md); this exits 0 by design and is"
        echo "  NOT a green gate. This run establishes nothing about the port."
        return 0
    fi
    if [ "$_d_skip" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_skip of $_d_n lanes skipped while $_d_pass ran."
        echo "  Toolchain absence skips EVERY lane. A partial skip means a lane declined"
        echo "  for a reason of its own, which is a fault, not a degradation:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") printf "    %-20s %s\n", $1, $2}' "$_d_res"
        return 1
    fi
    for _g in SWEEP CROSS; do
        if [ "$(gen_tuple "$_g" | cut -d'|' -f1)" = "SKIP" ]; then
            echo
            echo "GATE: FAIL — $_d_pass lanes graded a corpus and $(gen_name "$_g") skipped."
            echo "  The lanes found a toolchain, so its absence is not a degradation."
            return 1
        fi
    done

    echo
    # `--lane` filters the registry, and every check above then treats the filtered
    # list AS the registry — which is right, and which also means a one-lane run
    # can print `12/12 lanes ran` shaped exactly like a full gate. Same hole as an
    # unqualified PASS over a sampled sweep, on the half that already existed.
    if [ -n "$_d_filt" ]; then
        echo "GATE: PASS (LANES FILTERED) — $_d_pass/$_d_n SELECTED lanes ran, out of"
        echo "  $_d_filt in the registry. --lane is for iterating; this run says nothing"
        echo "  about the lanes it did not run. Re-run without --lane before reporting."
        if [ "$_d_swv" = "SAMPLED" ]; then
            echo "  The sweep was also SAMPLED — $_d_swc of $_d_swtot generated cases."
        fi
        if [ "$_d_cxv" = "SAMPLED" ]; then
            echo "  The cross was also SAMPLED — $_d_cxsel of $_d_cxtot case-lane cells."
        fi
        return 0
    fi
    if [ "$_d_swv" = "SAMPLED" ] || [ "$_d_cxv" = "SAMPLED" ]; then
        # A sample is a legitimate way to iterate and an illegitimate way to
        # report. It exits 0 and it does NOT get to print an unqualified PASS —
        # same treatment as GATE: SKIPPED, for the same reason.
        echo "GATE: PASS (SAMPLED) — $_d_pass/$_d_n lanes ran and every one of them graded"
        echo "  a corpus, but a generated instrument graded only part of its corpus:"
        [ "$_d_swv" = "SAMPLED" ] && \
            echo "    expr-sweep  $_d_swc of $_d_swtot generated cases"
        [ "$_d_cxv" = "SAMPLED" ] && \
            echo "    mode-cross  $_d_cxsel of $_d_cxtot case-lane cells"
        echo "  A strided sample is unbiased across fragments and is still a sample: this"
        echo "  run does NOT establish what a full run establishes. Re-run without"
        echo "  --sweep-cases / --cross-cells before reporting or landing."
        return 0
    fi
    echo "GATE: PASS — $_d_pass/$_d_n lanes ran and every one of them graded a corpus,"
    echo "  the sweep graded ${_d_swg:-?} of $_d_swtot generated cases and the cross graded"
    echo "  ${_d_cxg:-?} of $_d_cxtot case-lane cells, with 0 mismatches anywhere"
    echo "  (${_d_swu:-?} sweep cases carried ungraded — the reference rejects the source)."
    return 0
}

# --------------------------------------------------------------------------------
# Registry load + `--lane` filter, shared by every mode.
# --------------------------------------------------------------------------------
mkdir -p "$work"
reg="$work/registry.tsv"
parse_registry "$registry" "$reg"

# `--lane` filters the registry, and the filtered list then IS the registry for
# every check above — so `--lane` naming nothing is an empty registry and a hard
# error, never a run of zero lanes that exits 0.
filtered=""
if [ -n "$want" ]; then
    filtered=$(wc -l < "$reg")
    sel="$work/selected.tsv"; : > "$sel"
    for w in $want; do
        if ! awk -F"$TAB" -v w="$w" '$1==w{found=1} END{exit !found}' "$reg"; then
            echo "FATAL: --lane '$w' is not in $registry. Known lanes:" >&2
            cut -f1 "$reg" | sed 's/^/    /' >&2
            exit 2
        fi
        awk -F"$TAB" -v w="$w" '$1==w' "$reg" >> "$sel"
    done
    reg="$sel"
fi
nlanes=$(wc -l < "$reg")

case "$mode" in
list)
    echo "lane registry: $registry  ($nlanes lanes)"
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    exit 0 ;;
check)
    echo "lane registry: $registry"
    echo "  $nlanes lanes, slugs unique, every row parses."
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    exit 0 ;;
esac

if [ "$mode" = selftest ]; then
    # ----------------------------------------------------------------------------
    # Prove the gate fails when it should, with no toolchain and no compiler.
    # Every case fabricates lane logs and drives the REAL collect+decide path —
    # not a reimplementation of it, which would only prove the copy agrees.
    #
    # No command substitution anywhere in the loop: a `fails` counter incremented
    # inside `$(...)` lives in a subshell and is discarded, which would make this
    # selftest itself an instrument that reports green from an absence.
    # ----------------------------------------------------------------------------
    st="$work/selftest"; rm -rf "$st"; mkdir -p "$st"
    printf 'A\t/O1\nB\t/O1 /EHsc\n' > "$st/reg.tsv"
    fails=0
    cases=0
    CASE_DIR=""
    # The generated-instrument verdicts every lane case is driven with, so those
    # cases keep testing exactly what they tested before. Instrument-specific
    # cases override them. Eight fields: the last two are `graded` and `ungraded`,
    # added 2026-08-04 when the sweep was found to be counting 96 cases the oracle
    # never ruled on inside `checked`.
    SWEEP_OK='PASS|14635|14635|14635|0||14539|96'
    SWEEP_FOR_CASE="$SWEEP_OK"
    CROSS_OK='PASS|61539|61539|61539|0||61151|388'
    CROSS_FOR_CASE="$CROSS_OK"

    check_that() {  # <label> <ok?0/1>
        if [ "$2" -eq 0 ]; then
            printf '        %s\n' "also: $1"
        else
            printf '  FAIL  %s\n' "also: $1"
            fails=$((fails + 1))
        fi
    }

    run_case() {  # <name> <PASS|FAIL> <slug=body>...
        _rc_name="$1"; _rc_want="$2"; shift 2
        CASE_DIR="$st/$_rc_name"
        rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        for _rc_spec in "$@"; do
            _rc_slug=${_rc_spec%%=*}
            _rc_body=${_rc_spec#*=}
            case "$_rc_body" in
                MISSING) : ;;
                NOLINE)
                    echo "grading 197 fixtures at /O1" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
                *)
                    printf '%s\n' "$_rc_body" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
            esac
        done
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _rc_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" "$SWEEP_FOR_CASE" "" \
                "$CROSS_FOR_CASE" > "$CASE_DIR/out.txt" 2>&1; then
            _rc_got=FAIL
        fi
        SWEEP_FOR_CASE="$SWEEP_OK"
        CROSS_FOR_CASE="$CROSS_OK"
        _rc_hdl=$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo 'GATE: <none printed>')
        cases=$((cases + 1))
        if [ "$_rc_got" = "$_rc_want" ]; then
            printf '  ok    %-32s %s\n' "$_rc_name" "$_rc_hdl"
        else
            printf '  FAIL  %-32s wanted %s, got %s — %s\n' "$_rc_name" "$_rc_want" "$_rc_got" "$_rc_hdl"
            fails=$((fails + 1))
        fi
    }
    saw()    { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 0; else check_that "$2" 1; fi; }
    saw_no() { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 1; else check_that "$2" 0; fi; }

    P='LANE-RESULT PASS flags=[/O1 /GS- /c] graded=197 total=197 match=91 mismatch=0'
    M='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=1'
    V='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    S='LANE-RESULT SKIP flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L1='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L2='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=3'

    echo "gate.sh --selftest: driving the real collect+decide with fabricated lane logs"
    echo

    run_case both-pass PASS "A=$P" "B=$P"
    saw 'GATE: PASS' 'a wholly green run does say PASS'

    run_case lane-B-mismatch FAIL "A=$P" "B=$M"
    saw '^    B ' 'the failing lane is NAMED'
    saw 'ALARM'   'a mismatch raises the alarm banner'

    run_case lane-B-vacuous              FAIL "A=$P" "B=$V"
    run_case lane-B-no-log-at-all        FAIL "A=$P" "B=MISSING"
    saw 'NO RESULT' 'a lane that never ran is NO RESULT, not a pass'

    run_case lane-B-exit-0-no-result     FAIL "A=$P" "B=NOLINE"
    saw 'NO RESULT' 'exit 0 alone is not evidence a lane ran'

    run_case lane-B-lies-graded-0        FAIL "A=$P" "B=$L1"
    run_case lane-B-lies-with-mismatch   FAIL "A=$P" "B=$L2"
    run_case both-absent-is-not-a-skip   FAIL "A=MISSING" "B=MISSING"

    SWEEP_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    CROSS_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    run_case all-skip PASS "A=$S" "B=$S"
    saw    'GATE: SKIPPED' 'all-skip says SKIPPED and that nothing was graded'
    saw_no 'GATE: PASS'    'all-skip never says PASS'

    run_case partial-skip FAIL "A=$P" "B=$S"

    # ---- the SWEEP row (board #232) --------------------------------------------
    # Every one of these drives the real `sweep_verdict` + `decide` path. The gate
    # had ZERO references to the sweep until 2026-08-04, and a check nobody has
    # seen fail is not evidence of anything.

    # Drives ONE generated-instrument row from a fabricated log through the real
    # `sweep_verdict` + `decide`. `which` selects the row, so the sweep and the
    # cross are proved against the SAME rules rather than against two copies.
    gen_case() {  # <which:sweep|cross> <name> <PASS|FAIL> <log-body-or-MISSING> [exit]
        _sc_w="$1"; _sc_name="$2"; _sc_want="$3"; _sc_body="$4"; _sc_st="${5:-0}"
        CASE_DIR="$st/$_sc_name"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
        printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
        _sc_log="$CASE_DIR/$_sc_w.log"
        if [ "$_sc_body" = MISSING ]; then
            rm -f "$_sc_log"
        else
            printf '%s\n' "$_sc_body" > "$_sc_log"
        fi
        _sc_v=$(sweep_verdict "$_sc_log" "$_sc_st")
        if [ "$_sc_w" = sweep ]; then
            _sc_sw="$_sc_v"; _sc_cx="$CROSS_OK"
        else
            _sc_sw="$SWEEP_OK"; _sc_cx="$_sc_v"
        fi
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _sc_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$_sc_sw" "" \
                "$_sc_cx" > "$CASE_DIR/out.txt" 2>&1; then
            _sc_got=FAIL
        fi
        _sc_hdl=$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo 'GATE: <none printed>')
        cases=$((cases + 1))
        if [ "$_sc_got" = "$_sc_want" ]; then
            printf '  ok    %-32s %s\n' "$_sc_name" "$_sc_hdl"
        else
            printf '  FAIL  %-32s wanted %s, got %s — %s\n' \
                "$_sc_name" "$_sc_want" "$_sc_got" "$_sc_hdl"
            fails=$((fails + 1))
        fi
    }
    sweep_case() { gen_case sweep "$@"; }
    cross_case() { gen_case cross "$@"; }

    SW_FULL='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=14539 ungraded=96 unknown=0'
    SW_MISM='sweeping 14635 of 14635 generated cases
MISMATCH  /t/62-ctor-base-delegation-0032.cpp  |  struct Bd { Bd(); ~Bd(); int b0; };
checked=14635 mismatches=1 graded=14539 ungraded=95 unknown=0'
    SW_SHORT='sweeping 14635 of 14635 generated cases
checked=9107 mismatches=0 graded=9011 ungraded=96 unknown=0'
    SW_VAC='sweeping 0 of 14635 generated cases
checked=0 mismatches=0 graded=0 ungraded=0 unknown=0'
    SW_SAMP='sweeping 400 of 14635 generated cases (STRIDE 37 — a SAMPLE, not the corpus)
checked=400 mismatches=0 graded=397 ungraded=3 unknown=0'
    SW_NOCOUNT='sweeping 14635 of 14635 generated cases'
    SW_NOSWEEP='checked=14635 mismatches=0 graded=14539 ungraded=96 unknown=0'
    SW_SKIP='SKIP: toolchain absent — the sweep would be vacuous'
    # The pre-2026-08-04 count line. `checked` was reported as though it were
    # `graded`, and 96 of 14,635 cases had no reference obj at all — `c2rs diff`
    # prints four verdicts and the sweep's `case` recognized one. A log in the old
    # shape is NO-RESULT, not a log saying `ungraded=0`.
    SW_OLDLINE='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0'
    # Every case reached, NONE ruled on by the oracle. `mismatches=0` over
    # `graded=0` is the vacuous green this whole file exists to forbid, and it was
    # unreachable before `graded` existed as a separate number.
    SW_ALLUNGRADED='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=0 ungraded=14635 unknown=0'
    # A verdict string the classifier does not enumerate. That is how the 96 hid:
    # they fell out of a `case` with no default arm and were counted as clean.
    SW_UNKNOWN='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=14634 ungraded=0 unknown=1'

    CX_FULL='sweeping 61539 of 61539 case-lane cells
checked=61539 mismatches=0 graded=61151 ungraded=388 unknown=0'
    CX_MISM='sweeping 61539 of 61539 case-lane cells
MISMATCH  [/O1 /EHsc /GS- /c]  63-emit-order-0117.cpp
checked=61539 mismatches=1 graded=61151 ungraded=388 unknown=0'
    CX_SHORT='sweeping 61539 of 61539 case-lane cells
checked=47000 mismatches=0 graded=46700 ungraded=300 unknown=0'
    CX_SAMP='sweeping 4000 of 61539 case-lane cells (STRIDE 16 — a SAMPLE, not the product)
checked=4000 mismatches=0 graded=3975 ungraded=25 unknown=0'
    CX_SKIP='SKIP: toolchain absent — the cross would be vacuous'
    CX_NOCOUNT='sweeping 61539 of 61539 case-lane cells'

    sweep_case sweep-clean            PASS "$SW_FULL"
    saw 'GATE: PASS' 'a full clean sweep does say PASS'
    saw '14539 of 14635' 'the PASS line carries the sweep GRADED count over the corpus'

    sweep_case sweep-mismatch         FAIL "$SW_MISM" 1
    saw 'ALARM' 'a sweep mismatch raises the alarm banner'
    saw '62-ctor-base-delegation-0032' 'the failing CASE is named, by file'

    sweep_case sweep-short-count      FAIL "$SW_SHORT"
    saw 'SHORT' 'grading fewer cases than were selected is a FAIL, not a pass'

    sweep_case sweep-vacuous          FAIL "$SW_VAC"
    sweep_case sweep-no-checked-line  FAIL "$SW_NOCOUNT" 137
    saw 'NO RESULT' 'a sweep that died mid-run is NO RESULT, not 0 mismatches'

    sweep_case sweep-no-sweeping-line FAIL "$SW_NOSWEEP"
    saw 'NO RESULT' 'a checked= line with no corpus count is not a result'

    sweep_case sweep-log-missing      FAIL MISSING
    sweep_case sweep-clean-but-exit-1 FAIL "$SW_FULL" 1

    sweep_case sweep-sampled          PASS "$SW_SAMP"
    saw    'GATE: PASS (SAMPLED)' 'a sampled sweep says so in the headline'
    saw_no '^GATE: PASS —' 'a sampled sweep never prints an unqualified PASS'

    # Lanes green, sweep skipped: the partial-skip rule across the gate's halves.
    sweep_case sweep-skip-while-lanes-ran FAIL "$SW_SKIP"

    # ---- the three holes the sweep's own classifier had (w-modes) -------------
    sweep_case sweep-pre-graded-count  FAIL "$SW_OLDLINE"
    saw 'NO RESULT' 'a count line with no graded= is NO RESULT, not ungraded=0'
    sweep_case sweep-none-graded       FAIL "$SW_ALLUNGRADED"
    saw 'NONE graded' 'every case reached and none GRADED is vacuous, not clean'
    sweep_case sweep-unknown-verdict   FAIL "$SW_UNKNOWN"
    saw 'UNRECOGNIZED' 'an unenumerated verdict fails instead of counting as clean'

    # ---- the MODE-CROSS row (w-order Y-a) -------------------------------------
    # Same rules, same `sweep_verdict`, driven through the second row. The cross
    # is the only instrument that can see a defect needing a shape the fixtures
    # do not have AND a flag the sweep does not run.
    cross_case cross-clean             PASS "$CX_FULL"
    saw 'GATE: PASS' 'a full clean cross does say PASS'
    saw '61151 of 61539' 'the PASS line carries the cross GRADED count over the product'

    cross_case cross-mismatch          FAIL "$CX_MISM" 1
    saw 'ALARM' 'a cross mismatch raises the alarm banner'
    saw '63-emit-order-0117' 'the failing CASE is named, by file'
    saw 'O1 /EHsc' 'and the LANE it mismatched at is named too'

    cross_case cross-short-count       FAIL "$CX_SHORT"
    saw 'SHORT' 'grading fewer cells than were selected is a FAIL, not a pass'

    cross_case cross-log-missing       FAIL MISSING
    cross_case cross-no-checked-line   FAIL "$CX_NOCOUNT" 137
    cross_case cross-clean-but-exit-1  FAIL "$CX_FULL" 1

    cross_case cross-sampled           PASS "$CX_SAMP"
    saw    'GATE: PASS (SAMPLED)' 'a sampled cross says so in the headline'
    saw_no '^GATE: PASS —'        'a sampled cross never prints an unqualified PASS'

    cross_case cross-skip-while-lanes-ran FAIL "$CX_SKIP"

    # And the one the cross row exists for: no cross verdict AT ALL must fail,
    # even with every lane green and a clean sweep. This is the pre-w-modes gate,
    # which was green over w-order's Y-a for its whole life.
    CASE_DIR="$st/cross-absent"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" "" "" \
            > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a run with NO cross verdict PASSED\n' cross-absent
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' cross-absent "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # All lanes skipped but the cross reported a result: the partial-skip rule
    # across the gate's THIRD part, which did not exist before this row.
    CASE_DIR="$st/allskip-cross-ran"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$S" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$S" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" \
            'SKIP|0|0|0|0|toolchain absent' "" "$CROSS_OK" > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s all lanes skipped, cross ran, and it PASSED\n' allskip-cross-ran
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' allskip-cross-ran "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # And the one this whole addition exists for: no sweep verdict AT ALL must
    # fail, even with every lane green. This is the pre-2026-08-04 gate.
    CASE_DIR="$st/sweep-absent"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "" "" "$CROSS_OK" \
            > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a run with NO sweep verdict PASSED\n' sweep-absent
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' sweep-absent "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # A --lane-filtered run is a legitimate way to iterate and an illegitimate way
    # to report, exactly like a sampled sweep.
    CASE_DIR="$st/lanes-filtered"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" 12 \
            "$CROSS_OK" > "$CASE_DIR/out.txt" 2>&1; then
        if grep -q 'LANES FILTERED' "$CASE_DIR/out.txt" \
           && ! grep -q '^GATE: PASS —' "$CASE_DIR/out.txt"; then
            printf '  ok    %-32s %s\n' lanes-filtered "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
        else
            printf '  FAIL  %-32s a --lane run printed an unqualified PASS\n' lanes-filtered
            fails=$((fails + 1))
        fi
    else
        printf '  FAIL  %-32s a --lane run over green lanes did not exit 0\n' lanes-filtered
        fails=$((fails + 1))
    fi

    # The completeness assertion itself: a table short a row must fail even when
    # every row it does contain is a PASS.
    CASE_DIR="$st/short-table"; mkdir -p "$CASE_DIR"
    printf 'A\t/O1\tPASS|197|197|91|0|\n' > "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" "$SWEEP_OK" "" "$CROSS_OK" \
            > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a 1-row table for a 2-lane registry PASSED\n' short-table
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' short-table "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # An empty registry must be a hard error, not a run of zero lanes.
    : > "$st/empty.txt"
    cases=$((cases + 1))
    if parse_registry "$st/empty.txt" "$st/empty.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s an empty registry parsed clean\n' empty-registry
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused (a gate over 0 lanes cannot exist)\n' empty-registry
    fi

    # As must a duplicated slug.
    printf 'A /O1\nA /Ox\n' > "$st/dup.txt"
    cases=$((cases + 1))
    if parse_registry "$st/dup.txt" "$st/dup.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s a duplicated slug parsed clean\n' duplicate-slug
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused\n' duplicate-slug
    fi

    # As must a row that carries a slug and no flags. It used to be dropped, so
    # the registry silently held one lane fewer than the file listed.
    printf 'A /O1\nBroken\nC /Ox\n' > "$st/short.txt"
    cases=$((cases + 1))
    if parse_registry "$st/short.txt" "$st/short.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s a slug-only row was silently dropped (%s lanes from 3 rows)\n' \
            malformed-row "$(wc -l < "$st/short.tsv")"
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused (a row that does not parse is not a row that vanishes)\n' malformed-row
    fi

    # And the registry actually shipped must parse, and must carry an /EH lane —
    # the specific hole this whole registry was built to close.
    #
    # This is a deliberately WEAKER SUBSET of `crates/c2-harness/tests/
    # lane_registry.rs`, which is the binding assertion and is what `cargo test`
    # runs: that test additionally requires the /EHsc axis to be crossed over
    # EVERY base configuration, requires a lane that actually varies `/Oi`, and
    # requires the full lane count. This case is kept only so `--selftest` remains
    # self-contained on a machine with no cargo. It is a smoke check, not a second
    # definition of the rule — it cannot pass anything the test rejects, so the
    # two cannot drift in the direction that matters.
    cases=$((cases + 1))
    _n_real=$(wc -l < "$work/registry.tsv")
    _n_eh=$(cut -f2 "$work/registry.tsv" | grep -c -- '/EH' || true)
    if [ "$_n_real" -lt 2 ] || [ "$_n_eh" -lt 1 ]; then
        printf '  FAIL  %-32s %s lanes, %s of them /EH\n' shipped-registry "$_n_real" "$_n_eh"
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s lanes, %s of them compile /EH\n' shipped-registry "$_n_real" "$_n_eh"
    fi

    echo
    # The floor was 15 when the gate covered lanes only; the sweep row took it to
    # 27, and w-modes adds 3 sweep-classifier cases plus 10 mode-cross cases.
    # It is a floor on the COUNT, per the standing mitigation — compare a count,
    # never a status — and it must be raised whenever cases are added.
    if [ "$cases" -lt 40 ]; then
        echo "gate.sh --selftest: FAIL — only $cases cases ran; the selftest itself was"
        echo "  truncated, and a truncated selftest is the failure it exists to catch."
        exit 1
    fi
    if [ "$fails" -eq 0 ]; then
        echo "gate.sh --selftest: PASS — $cases cases, the gate fails on every one that should."
        exit 0
    fi
    echo "gate.sh --selftest: FAIL — $fails of $cases checks did not behave as required."
    exit 1
fi

# --------------------------------------------------------------------------------
# run
# --------------------------------------------------------------------------------
echo "lane gate: $nlanes lanes from $registry"
echo "  run dir: $work   (per-lane run dirs under $work/lanes/)"

# Pin ONE binary for the whole gate and hand it to every lane. Stronger than each
# lane pinning its own copy: all $nlanes lanes are then provably grading the same
# code, and the sha below is the answer to "which binary produced this table".
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$work"
export C2RS_BIN="$C2RS_PINNED"
export C2RS_MODE_LANE_WORK="$work/lanes"
: "${C2RS_JOBS:=8}"
export C2RS_JOBS

# Clear every lane's log and status FIRST, as its own pass, so that a result file
# which EXISTS is necessarily from this run. The `>` redirection below already
# truncates the log of any lane that is actually launched, so this is not covering
# a live hole; it closes the residual one — a lane the loop never launches at all
# (an interrupted run, a future `continue`, a re-run into a `--work` directory a
# previous run used) being graded from a previous run's log. That would be a stale
# PASS indistinguishable from a real one, which is the class `harness_bin.sh` was
# written to close one level down, and it is a two-line pass to make impossible.
while IFS="$TAB" read -r slug flags; do
    [ -n "$slug" ] || continue
    rm -f "$work/$slug.log" "$work/$slug.status"
done < "$reg"

started=$(date +%s)
running=0
while IFS="$TAB" read -r slug flags; do
    [ -n "$slug" ] || continue
    (
        st=0
        # shellcheck disable=SC2086
        sh "$repo_root/scripts/mode_lane.sh" $flags > "$work/$slug.log" 2>&1 || st=$?
        echo "$st" > "$work/$slug.status"
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$jobs" ]; then wait; running=0; fi
done < "$reg"
wait
elapsed=$(( $(date +%s) - started ))

collect "$reg" "$work" "$work/results.tsv"
echo
echo "wall clock: ${elapsed}s for $nlanes lanes at --jobs $jobs (C2RS_JOBS=$C2RS_JOBS)"

# --------------------------------------------------------------------------------
# The generated sweep — the other half of this gate (board #232).
#
# Run AFTER the lanes so a fast lane failure is on screen first, and always: there
# is no path through this file that reaches `decide` without a sweep verdict.
# `C2RS_BIN` is already exported, so `pin_harness` inside the sweep takes the
# gate's pinned copy and the two halves provably grade one binary.
# --------------------------------------------------------------------------------
unset C2RS_SWEEP_ONLY          # a filtered corpus is not a gate; see the header
export C2RS_SWEEP_JOBS="$jobs"
sweep_out="$work/sweep"
echo
echo "generated sweep: scripts/expr_sweep.sh $sweep_out ${sweep_cases}  (jobs $jobs)"
sw_started=$(date +%s)
sw_status=0
sh "$repo_root/scripts/expr_sweep.sh" "$sweep_out" "$sweep_cases" \
    > "$work/sweep.log" 2>&1 || sw_status=$?
tail -n 4 "$work/sweep.log" | sed 's/^/  /'
echo "  ($(( $(date +%s) - sw_started ))s)"
sweep_res=$(sweep_verdict "$work/sweep.log" "$sw_status")

# --------------------------------------------------------------------------------
# The MODE CROSS — the sweep's corpus x the lane registry (w-modes, 2026-08-04).
#
# The sweep grades 14,635 shapes at ONE profile, because `c2rs diff` hardcodes
# `/Ox /GS- /c`. The lanes grade 12 profiles over 228 hand-built shapes. w-order's
# **Y-a** was a live wrong emit that needed BOTH — an empty-bodied locally-defined
# unwind target *and* `/EHsc` — and it sat at `/O1 /EHsc`, the dc3 workload's own
# profile, where neither instrument could see it.
#
# The naive product is 175,620 gradings. `scripts/mode_invariance.py` measured how
# much of it is not redundant — 61,539 cells, 2.85x smaller — by proving, per
# fragment, which lanes hand the port a byte-identical IL bundle AND get a
# byte-identical obj back from c2 AND agree on `gy`. Those three together are the
# port's whole input and the oracle's whole output, so a merged lane is the same
# computation twice; `scripts/mode_classes.txt` is that table, every row keyed to a
# digest of its own fragment's cases so a generator edit un-applies the row rather
# than silently keeping a stale exclusion.
#
# It runs UNCONDITIONALLY, like the sweep, and for the same reason: there are
# twelve recorded instances on this project of an absence reading as a success, and
# an omittable check is an omitted check. `--cross-cells N` strides it for
# iteration and, like `--sweep-cases`, forfeits the unqualified PASS.
# --------------------------------------------------------------------------------
cross_out="$work/cross"
echo
echo "mode cross:      scripts/mode_cross.sh $cross_out ${cross_cells}  (jobs $jobs)"
cx_started=$(date +%s)
cx_status=0
C2RS_JOBS="$jobs" sh "$repo_root/scripts/mode_cross.sh" "$cross_out" "$cross_cells" \
    > "$work/cross.log" 2>&1 || cx_status=$?
grep -E '^(assigned|sweeping|checked=|SKIP:|FATAL|VACUOUS)' "$work/cross.log" \
    | sed 's/^/  /' || true
echo "  ($(( $(date +%s) - cx_started ))s)"
cross_res=$(sweep_verdict "$work/cross.log" "$cx_status")

decide "$reg" "$work/results.tsv" "$work" "$sweep_res" "$filtered" "$cross_res"
