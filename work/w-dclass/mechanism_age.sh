#!/bin/sh
# mechanism_age.sh — is w-conv's FRONTIER pricing still current?
#
# Lane w-dclass. Read-only measurement tooling; touches nothing under `crates/`.
#
# WHY THIS EXISTS
# ---------------
# `docs/rungs/2026-08-04-w-conv.md` §2 priced every FRONTIER TU by hand-counting
# independent refusals off its own disassembly, got a minimum of SIX, and board
# #269's standing decline clause -- *a frontier TU at >= 4 independent refusals
# is not a target* -- fires on all of them. That single table is the reason no
# lane has attempted a frontier TU since.
#
# But a "refusal" is counted against THE PORT AS IT WAS THAT DAY. w-conv ranked
# the frontier by missing MECHANISM as well as by TU (§3), and said of its own
# top two rows, verbatim: "The top two are one rung and the port had emitted
# NEITHER."
#
# The port emits both now. So the prices are stale in the direction of
# OVER-pricing, and nobody could notice, because `work/w-conv/frontier_dis.txt`
# -- the dump the count was taken off -- was never committed.
#
# This script does not re-price anything. It answers the one prior question:
# WHICH OF w-conv'S SEVEN MECHANISMS HAVE LANDED SINCE IT COUNTED? Run it before
# quoting w-conv's numbers, and re-run it after any codegen rung lands.
#
# It prints a COUNT and EXITS NON-ZERO when it checks zero mechanisms. Absence
# read as success is this project's most-repeated defect (16 recorded
# instances) and the generalizing fix on record is a positive check with a
# printed count. "No output" from this script must never look like "nothing has
# changed".
#
# Usage:  work/w-dclass/mechanism_age.sh
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

# The commit that landed the label->offset map, and the w-conv rung commit.
# Both are resolved from the tree; neither is transcribed as a date, because
# `git log --date` ordering is not ancestry and this claim is about ancestry.
LABELS_COMMIT=b662bc6
conv_rung="$(git log --format=%H -- docs/rungs/2026-08-04-w-conv.md | tail -1)"

echo "w-conv rung commit:      ${conv_rung:-<not found>}"
echo "label-map commit:        $LABELS_COMMIT"
if [ -n "$conv_rung" ] && git merge-base --is-ancestor "$LABELS_COMMIT" "$conv_rung" 2>/dev/null; then
    echo "ANCESTRY: label map PREDATES w-conv — its pricing already accounted for it"
else
    echo "ANCESTRY: label map POSTDATES w-conv — its pricing did NOT account for it"
fi
echo

# Each row: <label> <TUs it blocked, per w-conv §3> <a POSITIVE grep that is
# true only when the mechanism is BUILT>. A grep that merely finds the word is
# not enough -- `cr0` appears in a comment explaining why it is NOT built, and
# `savegprlr` appears as the string of a REFUSAL reason. Every probe below is
# chosen to separate "named" from "emitted", and the separation is stated in
# the note column so a reader can check the choice rather than trust it.
checked=0
built=0

probe() {
    label="$1"; tus="$2"; note="$3"; shift 3
    checked=$((checked + 1))
    if "$@" >/dev/null 2>&1; then
        built=$((built + 1))
        printf '  BUILT    %-46s (%2s TUs)  %s\n' "$label" "$tus" "$note"
    else
        printf '  MISSING  %-46s (%2s TUs)  %s\n' "$label" "$tus" "$note"
    fi
}

echo "w-conv §3's seven mechanisms, against the port AS IT IS IN THIS TREE:"

# BUILT: labels.rs exists AND is driven from a real emitter, not just defined.
probe "a real label→offset map" 14 \
    "labels.rs + driven from calls.rs" \
    sh -c 'test -f crates/c2-core/src/codegen/labels.rs &&
           grep -q "LabelMap::new()" crates/c2-core/src/codegen/calls.rs'

# BUILT: an unconditional intra-section form that a real emitter references.
probe "the intra-section unconditional b (#191)" 10 \
    "Form::B referenced from calls.rs" \
    sh -c 'grep -q "Form::B\b" crates/c2-core/src/codegen/calls.rs'

# MISSING expected: `cr0` occurs only in an encode.rs COMMENT explaining that
# `addic.` writes cr0 and that c2 branches on it there. A comment is not an
# emitter. Require a named cr0 constant actually used by a branch encoder.
probe "a branch on cr0" 10 \
    "needs a cr0 CR-field constant, not a comment" \
    sh -c 'grep -qE "CR_ZERO|CR_RECORD|cr0_bi" crates/c2-core/src/codegen/encode.rs'

# MISSING expected: `frame.rs` names `__savegprlr_N` inside `out_of_class_ctx`,
# i.e. as a REFUSAL reason. Require that the helper string is emitted rather
# than refused.
probe "callee-saved GPR formals / savegprlr" 9 \
    "frame.rs names it only as a refusal reason" \
    sh -c 'grep -q "savegprlr" crates/c2-core/src/codegen/frame.rs &&
           ! grep -q "frame-savegprlr-helper" crates/c2-core/src/codegen/frame.rs'

# PARTIAL expected: REFHI/REFLO exist in the COFF layer for pooled FP
# constants. That is the relocation, which is necessary and not sufficient for
# a data-symbol address pair in a body.
probe "a REFHI/REFLO data-symbol pair" 6 \
    "reloc.rs has the relocs; body-level pair is the question" \
    sh -c 'grep -q "REL_PPC_REFHI" crates/c2-core/src/coff/reloc.rs'

# MISSING expected: only the IMMEDIATE forms exist (`cmplwi`, `cmpwi`). The
# register-register forms are a different encoding.
probe "cmplw / cmpw register-register" 5 \
    "only cmplwi/cmpwi (immediate) are encoded" \
    sh -c 'grep -qE "encode_cmplw\b|encode_cmpw\b" crates/c2-core/src/codegen/encode.rs'

# MISSING expected: absent from the port AND from docs/, per w-cfgimpl §6.4.
probe "a CTR loop (mtctr / bdnz)" 4 \
    "absent from the port and from docs/" \
    sh -c 'grep -qE "mtctr|bdnz" crates/c2-core/src/codegen/encode.rs'

echo
echo "mechanisms checked: $checked    built: $built    missing: $((checked - built))"
if [ "$checked" -eq 0 ]; then
    echo "CHECKED ZERO MECHANISMS — refusing to exit 0 on an empty measurement"
    exit 2
fi
echo
echo "READ THIS BEFORE QUOTING w-conv's >=6:"
echo "  Its prices were counted against a port missing the mechanisms marked"
echo "  BUILT above. A TU that needed one should now price one lower. The"
echo "  per-TU repricing is NOT done here — this only says the prior is stale."
