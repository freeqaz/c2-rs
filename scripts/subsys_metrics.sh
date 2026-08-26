#!/bin/sh
# subsys_metrics.sh — regenerate `docs/SUBSYS_METRICS.md`, the per-subsystem
# scoreboard funded by `docs/DECISIONS_2026-08-22.md` decision 15.
#
# Lane `w-submetric`, boards #3617-#3622.
#
#   scripts/subsys_metrics.sh              print the console report
#   scripts/subsys_metrics.sh --write      regenerate docs/SUBSYS_METRICS.md
#   scripts/subsys_metrics.sh --keys       only the `subsys-metric` lines
#   scripts/subsys_metrics.sh --self-test  prove the verifier CAN go red
#
# ---------------------------------------------------------------------------
# WHAT THIS IS AND IS NOT
# ---------------------------------------------------------------------------
#
# The numbers are PROGRESS instruments under `docs/FUNCTION_BYTE_MATCH.md` §0,
# adopted verbatim: never in `scripts/gate.sh`'s verdict, their own block under
# their own disclaimer, namespaced keys, **they license no emit**, and a
# strength with no data prints a NAMED RESIDUE rather than 0.
#
# `#1406` binds any instrument whose output is quoted as evidence to run under
# `cargo test` or `scripts/gate.sh`. §0 forbids the second, so the resolution is
# `decode-reach`'s: the LOGIC and the CONTROLS live in `crates/c2-harness` and
# run under `cargo test --workspace` (a `gate.sh` row). This script is a thin
# wrapper over the same code so there is ONE producer of the table — it does not
# recompute anything itself, which is why it cannot drift from the tests.
#
# ---------------------------------------------------------------------------
# WHY --self-test EXISTS
# ---------------------------------------------------------------------------
#
# `#3336`: a control never seen failing is decoration. `--self-test` copies the
# whitebox reference index, corrupts it in three named ways, and REQUIRES the
# binary to refuse each one. If any corruption still verifies, this exits 3 and
# the scoreboard is not trustworthy — a green table over a broken input is
# exactly the failure mode the row was filed for.
#
# It needs no toolchain and no network: the corruptions are text edits on a
# copy under $TMPDIR, and the copy is removed on every exit path.

set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
doc="$root/docs/SUBSYS_METRICS.md"

# The release binary if it exists, otherwise a debug build. Never a stale one:
# a scoreboard printed by a binary older than the table is `#3128`'s defect
# family (a false green from a stale binary).
build_c2rs() {
    ( cd "$root" && cargo build --quiet -p c2-harness --bin c2rs "$@" )
}

c2rs_bin() {
    if [ "${C2RS_SUBSYS_RELEASE:-0}" = "1" ]; then
        build_c2rs --release
        echo "$root/target/release/c2rs"
    else
        build_c2rs
        echo "$root/target/debug/c2rs"
    fi
}

# ---------------------------------------------------------------------------
# --self-test
# ---------------------------------------------------------------------------
self_test() {
    bin="$(c2rs_bin)"
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/c2rs-subsys-selftest-XXXXXX")"
    trap 'rm -rf "$tmp"' EXIT INT TERM
    fails=0

    ref="$root/docs/whitebox/ref"

    # ---- CONTROL: the real index must VERIFY, or every red below is noise ---
    if "$bin" subsys --ref-dir "$ref" >"$tmp/clean.txt" 2>&1; then
        n="$(grep -c '^ *subsys-metric ' "$tmp/clean.txt" || true)"
        echo "  control: the real index verifies                     -> exit 0, $n keys"
    else
        echo "  CONTROL FAILED: the real reference index does not verify." >&2
        sed 's/^/    /' "$tmp/clean.txt" >&2
        fails=$((fails + 1))
    fi

    # A corruption case is only evidence if the MUTATION APPLIED. A sed that
    # matched nothing leaves a clean copy and the case "passes" by testing the
    # control twice — `#3516`'s mutation-not-applied failure, and
    # `gate_identity_diff.sh --self-test` names it in the same words.
    case_red() {
        _name="$1"; _dir="$2"; _want="$3"
        if diff -r -q "$ref" "$_dir" >/dev/null 2>&1; then
            echo "  FABRICATION DID NOT APPLY [$_name] — the case would test the control twice" >&2
            fails=$((fails + 1))
            return
        fi
        if "$bin" subsys --ref-dir "$_dir" >"$tmp/out.txt" 2>&1; then
            echo "  NOT CAUGHT [$_name] — a corrupted index still VERIFIED, exit 0" >&2
            fails=$((fails + 1))
        elif grep -q "$_want" "$tmp/out.txt"; then
            echo "  RED as required: $_name"
        else
            echo "  CAUGHT BUT BY THE WRONG CHECK [$_name] — wanted '$_want'" >&2
            grep -E '^ *- ' "$tmp/out.txt" | sed 's/^/    /' >&2
            fails=$((fails + 1))
        fi
    }

    # (1) A FUNCTION MOVES OUT OF A BAND -> the recount no longer reproduces.
    #     This is the corruption that matters: it is what a regenerated
    #     `FUNCS.tsv` would look like if Ghidra's analysis changed under the
    #     carried denominators.
    cp -r "$ref" "$tmp/r1"
    # 10b5b9de is a real FUNCS.tsv row inside the inliner band
    # 0x10b5b86d-0x10b62b00; move it out and the recount must drop 93 -> 92.
    sed -i 's/^10b5b9de\t/10a5b9de\t/' "$tmp/r1/FUNCS.tsv"
    case_red "a function moved out of the inliner band" "$tmp/r1" "DOES NOT REPRODUCE"

    # (2) A PAGE'S COVERAGE LINE MOVES -> the carried number is now unsourced.
    cp -r "$ref" "$tmp/r2"
    sed -i 's/19 entries against a denominator of 47/20 entries against a denominator of 47/' \
        "$tmp/r2/P_EH.md"
    case_red "P_EH.md's coverage line edited" "$tmp/r2" "den_probe not found"

    # (3) A SUBSYSTEM DISAPPEARS FROM SUBSYS.md §1 -> the scoreboard and the
    #     unit list have diverged, in the direction where the table still looks
    #     complete.
    cp -r "$ref" "$tmp/r3"
    sed -i '/P_GLOBREGS.md/d' "$tmp/r3/SUBSYS.md"
    case_red "a subsystem dropped from SUBSYS.md §1" "$tmp/r3" "SUBSYS.md §1 has no row"

    if [ "$fails" -gt 0 ]; then
        echo "SELF-TEST FAIL: $fails case(s)." >&2
        return 3
    fi
    echo "SELF-TEST PASS: the real index verifies; all three corruptions go RED"
    echo "  through the check that owns them, and each mutation was proved applied."
    return 0
}

# ---------------------------------------------------------------------------
# --write
# ---------------------------------------------------------------------------
write_doc() {
    bin="$(c2rs_bin)"
    tip="$(cd "$root" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"
    dirty=clean
    if [ -n "$(cd "$root" && git status --porcelain 2>/dev/null)" ]; then dirty=DIRTY; fi
    now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    tmp="$(mktemp "${TMPDIR:-/tmp}/c2rs-subsys-XXXXXX")"
    trap 'rm -f "$tmp"' EXIT INT TERM

    # The body is the ONE producer's own output (`c2rs subsys --markdown`); this
    # script contributes only the generation stamp, so the doc cannot drift from
    # the code that grades it.
    {
        "$bin" subsys --markdown | sed '1,/^$/!b' | head -1
        echo
        echo "> **GENERATED — do not hand-edit.** Regenerate with"
        echo "> \`scripts/subsys_metrics.sh --write\`. Tree \`$tip\` ($dirty), generated"
        echo "> \`$now\`. Every number below is re-verified against this tree by"
        echo "> \`cargo test -p c2-harness --lib subsys\`, which \`scripts/gate.sh\`'s"
        echo "> unit row runs; the four positive controls run beside it."
        echo
        "$bin" subsys --markdown | tail -n +2
    } > "$tmp"

    mv "$tmp" "$doc"
    trap - EXIT INT TERM
    echo "wrote $(basename "$doc") ($(wc -l < "$doc") lines) at tree $tip ($dirty)"
}

case "${1:-}" in
    --self-test) self_test ;;
    --write)     write_doc ;;
    --keys)      exec "$(c2rs_bin)" subsys --keys ;;
    --markdown)  exec "$(c2rs_bin)" subsys --markdown ;;
    "")          exec "$(c2rs_bin)" subsys ;;
    *)           echo "usage: $0 [--write | --keys | --markdown | --self-test]" >&2; exit 2 ;;
esac
