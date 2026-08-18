#!/bin/sh
# run.sh — compile the frozen candorder grid at BOTH profiles, keep the obj and
# the /FAsc listing for each, and assert the batch actually executed.
#
# Lane w-dagorder (WB-DAGORDER2). Read-only w.r.t. crates/.
#
# Probe soundness (docs/rungs/README.md, 2026-08-17, boards #3219/#3231): a
# fresh worktree has no compilers/, so capture work SILENTLY SKIPS and looks
# successful. This script therefore:
#   - runs a control PINNED BY NAME (fixtures/cpp/w5_chain.cpp -> 4/4) first,
#   - asserts a nonzero obj for every cell rather than trusting an exit code,
#   - prints the wall time, because a 0.00 s batch is void.
set -eu

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out_root="$repo_root/work/w-dagorder/ref"

sib() {
    d="$repo_root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}

wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"

[ -x "$wibo" ] || { echo "VOID: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "VOID: toolchain absent (cl.exe)"; exit 3; }

grid="$repo_root/docs/whitebox/grids/wb-dagorder2/candorder_grid.cpp"
want=b06a05fc83fe0cca45a539684b87d88998b70c305e28ca29b54fb4fcefafeee6
got=$(sha256sum "$grid" | cut -d' ' -f1)
[ "$got" = "$want" ] || { echo "VOID: grid hash $got != frozen $want"; exit 1; }
echo "grid hash OK: $want"

start=$(date +%s%N)

# ---- the control, pinned by NAME, in THIS environment -----------------------
ctl=$("$repo_root/target/release/c2rs" census "$repo_root/fixtures/cpp/w5_chain.cpp" 2>&1 | grep 'functions in class')
echo "CONTROL w5_chain.cpp: $ctl"
printf '%s' "$ctl" | grep -q '4/4' || { echo "VOID: control did not read 4/4"; exit 1; }

# ---- the two profiles -------------------------------------------------------
# O1 is the workload's own, read from flags.txt minus its -I set (the grid has
# no includes) so it cannot drift from what c2rs gap grades.
o1="/nologo /c /GR /O1 /Oi /EHsc"
ox="/nologo /c /GR /Ox /Oi /EHsc"

for prof in o1 ox; do
    eval "flags=\$$prof"
    d="$out_root/$prof"
    rm -rf "$d"; mkdir -p "$d"
    cp "$grid" "$d/g.cpp"
    zo="Z:$(printf '%s' "$d/g.obj" | tr '/' '\\')"
    za="Z:$(printf '%s' "$d/g.asm" | tr '/' '\\')"
    ( cd "$d" && TMP="$d" TEMP="$d" WIBO_FS_CACHE=1 \
        "$wibo" "$cl" $flags "/FAsc" "/Fa$za" "/Fo$zo" "g.cpp" \
        >"$d/cl.log" 2>&1 || true )
    [ -s "$d/g.obj" ] || { echo "VOID: no obj at $prof"; exit 1; }
    [ -s "$d/g.asm" ] || { echo "VOID: no listing at $prof"; exit 1; }
    echo "$prof: obj $(stat -c%s "$d/g.obj") B, listing $(stat -c%s "$d/g.asm") B, flags: $flags"
done

end=$(date +%s%N)
ms=$(( (end - start) / 1000000 )); echo "batch wall time: ${ms} ms   (a 0 ms batch is VOID)"; [ "$ms" -gt 0 ] || { echo "VOID: zero-duration batch"; exit 1; }
