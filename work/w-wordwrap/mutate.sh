#!/bin/sh
# One MUST-FAIL mutation: break one clause of `global_store_leaf`, rebuild, and
# re-grade this lane's own `_neg` cells with the BYTE judge.
#
# **The grading is `fnbyte-*`, not the TU verdict**, and that is forced rather
# than chosen: the class's object is a non-COMDAT `.bss` the writer refuses by
# name, so no fixture of this class can reach `match` and the verdict column
# says nothing. `fnbyte` is still the oracle — real c2's own obj, bytes AND all
# four relocation records — so a mutation that admits a `_neg` cell shows up as
# `differs`, which is the same evidence a `mismatch` would be.
#
#     work/w-wordwrap/mutate.sh M1 'sed -e …'
#
# ALWAYS run from a COMMITTED tree (#2668/#2699: a restore trap discards
# uncommitted `crates/` edits while printing that the diff is empty). This
# script refuses on a dirty tree rather than trusting itself to restore one.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
shift
if [ -n "$(git -C "$repo" status --porcelain -- crates)" ]; then
    echo "REFUSING: crates/ is dirty. Commit first (#2668)." >&2
    exit 2
fi
"$@"
cargo build --release --manifest-path "$repo/Cargo.toml" >/dev/null 2>&1 || {
    echo "$tag BUILD FAILED"
    git -C "$repo" checkout -- crates
    exit 1
}
"$here/fxbyte.sh" "/O1 /Oi" "mut-$tag" \
    fixtures/cpp/wwrap_gstore.cpp \
    fixtures/cpp/wwrap_gstore_widths.cpp \
    fixtures/cpp/wwrap_gstore_conv_neg.cpp \
    fixtures/cpp/wwrap_gstore_lit_neg.cpp \
    fixtures/cpp/wwrap_gstore_two_neg.cpp \
    fixtures/cpp/wwrap_gstore_second_neg.cpp \
    fixtures/cpp/wwrap_gstore_float_neg.cpp \
    fixtures/cpp/wwrap_gstore_sub_neg.cpp \
    fixtures/cpp/wwrap_gstore_gg_neg.cpp
git -C "$repo" checkout -- crates
# **And the restore is VERIFIED**, not assumed — #2699's lane lost the fix it
# was written to grade to exactly this step.
if [ -n "$(git -C "$repo" status --porcelain -- crates)" ]; then
    echo "$tag RESTORE FAILED — crates/ is still dirty" >&2
    exit 2
fi
cargo build --release --manifest-path "$repo/Cargo.toml" >/dev/null 2>&1
