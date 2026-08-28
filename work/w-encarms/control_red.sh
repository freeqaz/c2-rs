#!/bin/sh
# Lane `w-encarms` — watch every control FAIL before any verdict from it is
# quoted (`#3336`: a control nobody has seen fail is decoration).
#
#   run:  sh work/w-encarms/control_red.sh   (from the repo root)
#
# Each block plants a defect, shows the check go RED, restores, shows GREEN.
set -e
cd "$(dirname "$0")/../.."
H=work/w-encarms/armhist.py

echo "=== C-0  baseline: the instrument's own self-test"
python3 "$H" --self-test; echo "   exit=$?"

echo
echo "=== C-A  PLANT: widen the first mask over the RB field (0xFC0007FF -> 0xFC00FFFF)"
echo "         so a word with a non-zero RB can no longer be attributed."
cp "$H" "$H.bak"
sed -i 's/0xFC0007FF/0xFC00FFFF/' "$H"
set +e
python3 "$H" --self-test; echo "   exit=$?  <-- MUST be 1"
set -e
mv "$H.bak" "$H"
echo "         RESTORED:"
python3 "$H" --self-test; echo "   exit=$?  <-- MUST be 0"

echo
echo "=== C-B  PLANT: P3 says arm 10bfa81d (the ICE arm, forms 8/9/10/11/13/48/60)"
echo "         is reached ZERO times.  Show the count CAN be non-zero: re-point"
echo "         a high-frequency opcode (\`or\`, 0x011d) at that arm and re-count."
python3 - <<'PY'
import importlib.util, json, os, sys
spec = importlib.util.spec_from_file_location("armhist", "work/w-encarms/armhist.py")
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
rows = m.load_table("docs/whitebox/ref/ENCODE_OPCODES.txt")
objs = "/home/free/code/milohax/dc3-decomp/build/373307D9"
srcs = [json.loads(l)["src"] for l in open("work/w-bss/census/sections.jsonl")]
paths = [os.path.join(objs, s.rsplit(".", 1)[0] + ".obj") for s in srcs]
paths = [p for p in paths if os.path.exists(p)][:40]   # 40 objs is enough to move it


def count(tbl, arm):
    idx = m.build_index(tbl)
    zero = {o for o, (_x, bw, _f, _a) in tbl.items() if bw == 0}
    n = 0
    for p in paths:
        data = open(p, "rb").read()
        for w, _rt in m.text_words(data):
            if w == 0:
                continue
            _mk, cand = m.attribute(w, idx)
            cand = [o for o in cand if o not in zero]
            if cand and arm in {tbl[o][3] for o in cand}:
                n += 1
    return n


asread = count(rows, "10bfa81d")
mut = dict(rows)
mn, bw, form, _arm = mut[0x011D]
mut[0x011D] = (mn, bw, form, "10bfa81d")
planted = count(mut, "10bfa81d")
print(f"   as read : 10bfa81d reached {asread} times over {len(paths)} objs   <-- P3 predicts 0")
print(f"   planted : 10bfa81d reached {planted} times over the same objs")
if asread != 0 or planted == 0:
    print("   C-B FAILED: the check cannot distinguish the two")
    sys.exit(1)
print("   C-B OK: the count is capable of being non-zero, and as read it is 0")
PY

echo
echo "=== all controls watched"
