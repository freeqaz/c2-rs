#!/bin/sh
# M2 is TWO edits, applied together on purpose: the `2C` clause and the
# type-restatement clause BOTH refuse `conv_neg`, so deleting either alone comes
# back green for the wrong reason (#2665/#2698 — a merged clause's must-fail
# mutation must delete the whole conjunction).
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
python3 "$here/mut.py" M2
python3 "$here/mut.py" M2b
