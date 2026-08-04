#!/bin/sh
# w_gr_counterfactual.sh — **can the new RTTI cases FAIL?**
#
# A fragment that only ever prints `NotImplemented` and `Match` is
# indistinguishable from a fragment that grades nothing, and `docs/STATUS.md`
# trap 5 says which of those a reader will assume. So this script breaks the
# port on purpose, measures how many cases turn into `Port=Mismatch`, restores
# the tree, and **proves the restore** — the last step matters as much as the
# first, because a breaker left in the tree is a wrong emit somebody else has to
# find.
#
# ---- the breaker, and why THIS one ---------------------------------------------
#
# `encode_lwz(rd, ra, d)` (`crates/c2-core/src/codegen/encode.rs`) loses 4 from
# every displacement of 4 or more. In one sentence: **the port forgets the
# vfptr.**
#
# That is not an arbitrary corruption. A polymorphic class puts its vftable
# pointer at offset 0, so `struct S{S();virtual ~S();int s;}` has `s` at +4 where
# its non-virtual twin has it at +0; multiple inheritance pushes the second
# base's members further; a virtual base adds a displacement on top. The 27 cases
# of `91-rtti-vftable.py` that the port accepts today are almost all member loads
# through exactly those layouts. A breaker that says "ignore the vfptr" therefore
# lands on the axis this fragment was written for — and, because it leaves
# displacement 0 alone, it does **not** touch the large majority of the corpus,
# which is the property that makes the result mean something. A breaker that
# breaks everything proves nothing.
#
# The script reports the per-fragment histogram, unedited, including the other
# fragments it lights up. Those are a real part of the result: the honest claim
# is "this breaker is narrow and it reaches the new cases", not "this breaker is
# unique to the new cases", which would be false.
#
# ---- what a PASS looks like -----------------------------------------------------
#
#   * `91-rtti-vftable` mismatch count > 0                    (the cases can fail)
#   * total mismatch count > the count in `91-rtti-vftable`   (…and are not alone)
#   * some fragment with 0 mismatches                         (…and it is not uniform)
#   * `git status --porcelain crates/` empty at the end       (…and the tree is clean)
#
# Every one is a COUNT compared against a floor, never a status word.
#
# Usage:  scripts/w_gr_counterfactual.sh [outdir]
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$repo_root/work/w-gr/cf}"
target="$repo_root/crates/c2-core/src/codegen/encode.rs"
mkdir -p "$out"

# Refuse to start on a dirty crates/ — otherwise the restore below cannot be
# distinguished from "it was already modified", and the proof at the end is
# vacuous.
dirty="$(cd "$repo_root" && git status --porcelain crates/ | wc -l)"
if [ "$dirty" -ne 0 ]; then
    echo "REFUSING: crates/ has $dirty modified path(s) before the breaker is" >&2
    echo "  applied. The restore proof at the end could not tell them apart." >&2
    (cd "$repo_root" && git status --porcelain crates/) >&2
    exit 2
fi

if [ ! -x "${C2RS_WIBO:-$repo_root/../wibo/build/release/wibo}" ] \
   && ! command -v wibo >/dev/null 2>&1; then
    echo "SKIP: toolchain absent — the counterfactual would grade nothing"
    exit 0
fi

restore() {
    (cd "$repo_root" && git checkout -- crates/c2-core/src/codegen/encode.rs) || true
}
trap 'restore' EXIT INT TERM

# ---- 1. the baseline ------------------------------------------------------------
echo "==> generating the corpus"
rm -rf "$out/cases"; mkdir -p "$out/cases"
python3 "$repo_root/scripts/sweep_gen.py" "$out/cases" "$repo_root/scripts/sweep.d" \
    | tail -1

grade() {
    _tag="$1"
    cargo build --release --manifest-path "$repo_root/Cargo.toml" -p c2-harness 2>&1 | tail -1
    : > "$out/verdicts-$_tag.txt"
    ls "$out"/cases/*.cpp | xargs -P "${C2RS_JOBS:-8}" -I@ sh -c \
        'v=$('"$repo_root"'/target/release/c2rs diff "@" 2>&1 | tail -1); echo "@ | $v"' \
        >> "$out/verdicts-$_tag.txt"
    _n=$(wc -l < "$out/verdicts-$_tag.txt")
    _m=$(grep -c 'Port=Mismatch' "$out/verdicts-$_tag.txt" || true)
    echo "  $_tag: graded $_n cases, $_m mismatches"
    echo "$_m" > "$out/mism-$_tag"
}

echo "==> BASELINE (tree as committed)"
grade base

# ---- 2. the breaker -------------------------------------------------------------
echo "==> applying the breaker: encode_lwz loses 4 from every displacement >= 4"
python3 - "$target" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """pub fn encode_lwz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 ="""
new = """pub fn encode_lwz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let d = if d >= 4 { d - 4 } else { d };  // W-GR COUNTERFACTUAL — reverted
    let word: u32 ="""
assert s.count(old) == 1, "encode_lwz did not match its expected text — the breaker must be re-derived"
open(p, "w").write(s.replace(old, new))
PY
grep -q "W-GR COUNTERFACTUAL" "$target" || { echo "FATAL: breaker not applied" >&2; exit 3; }

echo "==> BROKEN"
grade broken

# ---- 3. restore, and PROVE it ---------------------------------------------------
restore
trap - EXIT INT TERM
clean="$(cd "$repo_root" && git status --porcelain crates/ | wc -l)"
echo "==> restored; git status --porcelain crates/ = $clean path(s)"
if [ "$clean" -ne 0 ]; then
    echo "FATAL: crates/ is still modified after the restore." >&2
    (cd "$repo_root" && git status --porcelain crates/) >&2
    exit 4
fi
grep -q "W-GR COUNTERFACTUAL" "$target" && { echo "FATAL: breaker still present" >&2; exit 4; }

# ---- 4. the numbers -------------------------------------------------------------
base_m=$(cat "$out/mism-base"); broken_m=$(cat "$out/mism-broken")
echo
echo "baseline mismatches: $base_m"
echo "broken   mismatches: $broken_m"
echo
echo "per-fragment mismatch histogram under the breaker:"
grep 'Port=Mismatch' "$out/verdicts-broken.txt" \
    | sed 's|.*/||; s/-[0-9]*\.cpp .*//' | sort | uniq -c | sort -rn
echo
echo "fragments with ZERO mismatches under the breaker (the breaker is NOT uniform):"
python3 - "$out" <<'PY'
import os, sys, collections
out = sys.argv[1]
allf = collections.Counter()
for n in os.listdir(os.path.join(out, "cases")):
    if n.endswith(".cpp"):
        allf[n.rsplit("-", 1)[0]] += 1
bad = collections.Counter()
for line in open(os.path.join(out, "verdicts-broken.txt")):
    if "Port=Mismatch" in line:
        bad[os.path.basename(line.split(" | ")[0]).rsplit("-", 1)[0]] += 1
zero = sorted(f for f in allf if not bad[f])
print("  %d of %d fragments: %s" % (len(zero), len(allf), " ".join(zero)))
mine = bad["91-rtti-vftable"]
print()
print("VERDICT (counts, not statuses):")
print("  91-rtti-vftable mismatches under the breaker : %d   (must be > 0)" % mine)
print("  total mismatches                             : %d   (must be > %d)"
      % (sum(bad.values()), mine))
print("  fragments untouched by the breaker           : %d   (must be > 0)" % len(zero))
ok = mine > 0 and sum(bad.values()) > mine and len(zero) > 0
print("  COUNTERFACTUAL: %s" % ("PASS" if ok else "FAIL"))
sys.exit(0 if ok else 1)
PY
