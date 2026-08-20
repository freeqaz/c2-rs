#!/bin/sh
# Generate the real-workload inputs for `c2rs gap` from a dc3-decomp checkout:
# the TU list and the project's real compile flags (with the original
# e:/lazer_build_gmc1 include roots mapped onto the local source tree, the
# same mapping dc3-decomp's own tooling uses).
#
# Usage:  scripts/gen_dc3_workload.sh [dc3-decomp-root]   (default: ../dc3-decomp)
# Writes: work/dc3-workload/{files.txt,flags.txt} and prints the scan command.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
dc3="${1:-$repo_root/../dc3-decomp}"
out="$repo_root/work/dc3-workload"

if [ ! -d "$dc3/src" ] || [ ! -f "$dc3/config/373307D9/config.json" ]; then
    echo "SKIP: no dc3-decomp checkout at $dc3 (pass its path as \$1)"
    exit 0
fi

mkdir -p "$out"

# Skip dotfiles (permuter scratch copies etc.) — only the tracked sources.
(cd "$dc3" && find src -name '*.cpp' ! -name '.*' | sort) > "$out/files.txt"

python3 - "$dc3" > "$out/flags.txt" << 'PY'
import json, sys
dc3 = sys.argv[1]
cfg = json.load(open(f"{dc3}/config/373307D9/config.json"))
flags = list(cfg["cflags"]["base"]["flags"])
ns = {}
exec(open(f"{dc3}/tools/defines_common.py").read(), ns)
mapped = 0
for flag in ns["cflags_includes"]:
    path = flag[3:] if flag.startswith("/I ") else flag
    # Original build roots -> local source tree (same mapping as dc3's tooling).
    #
    # MATCH ON A SEPARATOR-NORMALISED COPY. dc3's config writes these with
    # BACKSLASHES (`e:\lazer_build_gmc1\system\src`), and this loop used to
    # compare them against forward-slash literals, so every mapping silently
    # missed and the generated flags kept the Windows build root verbatim.
    # The failure is invisible here and total downstream: `c2rs gap` then
    # reports **851 of 878 TUs capture-fail (C1083)** with `mismatch 0`, which
    # reads like a workload that legitimately does not compile. Found by lane
    # `ir0`'s fix round, which regenerated this file and got that scan.
    probe = path.replace("\\", "/")
    for orig, local in (
        ("e:/lazer_build_gmc1/system/src", "src/system"),
        ("e:/lazer_build_gmc1/lazer/src", "src/lazer"),
    ):
        if probe.lower().startswith(orig):
            path = local + probe[len(orig):]
            mapped += 1
            break
    flags += ["/I", path]
# A generator whose whole job is a path rewrite must not succeed having
# rewritten nothing.
if mapped == 0:
    sys.exit("gen_dc3_workload.sh: NO include root was mapped onto the local "
             "source tree — the flags would keep e:\\lazer_build_gmc1\\... and "
             "the scan would be ~97% capture-fail. Check the roots above "
             "against dc3's tools/defines_common.py.")
print(" ".join(flags))
PY

n=$(wc -l < "$out/files.txt")
echo "wrote $out/files.txt ($n TUs) and flags.txt:"
sed 's/^/  /' "$out/flags.txt"
echo
echo "scan with:"
echo "  cargo run --release -p c2-harness --bin c2rs -- gap \\"
echo "    --list $out/files.txt --flags-file $out/flags.txt \\"
echo "    --cwd $dc3 --jsonl $out/scan.jsonl --replay-every 50"
