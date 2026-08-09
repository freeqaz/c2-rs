#!/bin/sh
# The BASE test count, measured on an EXACT base tree.
#
# Not a `git checkout <rev> -- crates` round trip with this lane's new files
# left lying around (that stages, #2512, and it leaves the new fixtures and the
# new test target in place so the "base" is not the base). The measured base is
# a TEMPORARY COMMIT of `crates/` and `fixtures/` at the merge-base with this
# lane's added files removed, verified with `git diff --stat`, then
# `git reset --hard` back to the saved tip.
#
# `docs/` is deliberately left at the tip: reverting it would delete the rung
# being written. The one failure that produces is reported, not netted out.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo"
BASE=5831a092
TIP="$(git rev-parse HEAD)"
echo "tip $TIP  base $BASE"
trap 'git reset --hard "$TIP" >/dev/null 2>&1; echo "restored to $TIP"' EXIT INT TERM

git checkout "$BASE" -- crates fixtures
git rm -q --cached --ignore-unmatch \
    crates/c2-harness/tests/pool2_cells.rs \
    crates/c2-il/src/func/body/shapes/pool_free_list.rs \
    crates/c2-il/src/func/body/shapes/pool_ctor_chain.rs \
    crates/c2-core/src/codegen/pool_free_list.rs \
    crates/c2-core/src/codegen/pool_ctor_chain.rs \
    fixtures/cpp/wpool2_free_list.cpp \
    fixtures/cpp/wpool2_free_list_neg.cpp >/dev/null 2>&1 || true
rm -f crates/c2-harness/tests/pool2_cells.rs \
      crates/c2-il/src/func/body/shapes/pool_free_list.rs \
      crates/c2-il/src/func/body/shapes/pool_ctor_chain.rs \
      crates/c2-core/src/codegen/pool_free_list.rs \
      crates/c2-core/src/codegen/pool_ctor_chain.rs \
      fixtures/cpp/wpool2_free_list.cpp \
      fixtures/cpp/wpool2_free_list_neg.cpp
git add -A crates fixtures
git -c user.name=tmp -c user.email=tmp@local commit -q -m "TEMPORARY base tree" || true
echo "diff vs base over crates+fixtures (must be 0 lines):"
git diff "$BASE" --stat -- crates fixtures | tail -2
echo "fixtures at base: $(ls fixtures/cpp/*.cpp | wc -l)"
cargo test --workspace --release --no-fail-fast > "$here/tests_base.log" 2>&1 || true
python3 - "$here/tests_base.log" <<'PY'
import re, sys
p = f = n = 0
for line in open(sys.argv[1]):
    m = re.match(r"test result: (\w+)\. (\d+) passed; (\d+) failed", line)
    if m:
        n += 1; p += int(m.group(2)); f += int(m.group(3))
print(f"BASE targets {n}  passed {p}  failed {f}")
PY
grep -B2 "FAILED" "$here/tests_base.log" | head -20 || true
