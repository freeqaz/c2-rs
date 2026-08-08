#!/bin/sh
# wb-chooser — run every grid cell. See run.sh for the one-cell driver.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
for c in m1 m2 m3 m4 m5 m6 m7 m8 m9 m10 m11 m12 m13 \
         b1 b2 b3 b4 b5 b6 b7 bp2 bp3 bp4 bp6; do
    sh "$here/run.sh" "$c" || echo "FAIL: $c"
done
