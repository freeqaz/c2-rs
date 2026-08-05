#!/bin/sh
# Construct a REAL resource-exhausted filesystem and drive the real gate at it.
#
# Not a fabricated log and not "the gate was green while /tmp happened to be at
# 16%". An unprivileged user namespace can mount a tmpfs with an arbitrary
# `nr_inodes` ceiling, so the condition can be built on demand rather than waited
# for -- which matters because the real condition is TRANSIENT and does not
# reproduce (coordinator, 2026-08-05).
set -eu
REPO="$1"
MNT="$REPO/work/w-ledger/tinyfs"
mkdir -p "$MNT"

unshare -Umr sh -s "$REPO" "$MNT" <<'INNER'
set -eu
REPO="$1"; MNT="$2"

run() {  # <label> <size> <nr_inodes> <files-to-burn>
    mount -t tmpfs -o "size=$2,nr_inodes=$3" tmpfs "$MNT"
    i=0
    while [ "$i" -lt "$4" ]; do touch "$MNT/burn.$i" 2>/dev/null || break; i=$((i+1)); done
    echo "=================================================================="
    echo "CASE: $1"
    df -kP "$MNT" | tail -1 | sed 's/^/  space  /'
    df -iP "$MNT" | tail -1 | sed 's/^/  inodes /'
    rc=0
    sh "$REPO/scripts/gate.sh" --work "$MNT/c2rs-gate-test" >"$MNT/../out.$1" 2>&1 || rc=$?
    echo "  --- gate.sh exit $rc ---"
    sed 's/^/  | /' "$MNT/../out.$1"
    umount "$MNT"
}

# 1. INODES gone, SPACE fine. This is w-alias's failure exactly: `df -h` looked
#    healthy at ~19 G free while `df -i` was at 1048576/1048576.
run inodes-gone 8G 200 190

# 2. SPACE gone, INODES fine. The mirror image, so the verdict is shown to name
#    the resource rather than to have one hardcoded answer.
dd_case=1; mount -t tmpfs -o "size=2M,nr_inodes=100000" tmpfs "$MNT"; dd if=/dev/zero of="$MNT/fill" bs=1K count=4096 2>/dev/null || true; echo "=================================================================="; echo "CASE: space-gone"; df -kP "$MNT" | tail -1 | sed "s/^/  space  /"; df -iP "$MNT" | tail -1 | sed "s/^/  inodes /"; rc=0; sh "$REPO/scripts/gate.sh" --work "$MNT/c2rs-gate-test" >"$MNT/../out.space-gone" 2>&1 || rc=$?; echo "  --- gate.sh exit $rc ---"; sed "s/^/  | /" "$MNT/../out.space-gone"; umount "$MNT"

# 3. TOTAL inode exhaustion -- not one inode left, so even `mkdir` fails.
run inodes-zero 8G 100 200
INNER
