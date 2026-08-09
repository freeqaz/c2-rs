#!/bin/sh
# M3 is TWO edits. The first run of this grid applied the arity fence alone and
# came back GREEN: `second_neg`'s value is params[1], so `val_tok != params[0]`
# refused it anyway. The two clauses are one conjunction over that cell (#2665),
# and the mutation deletes both.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
python3 "$here/mut.py" M3
python3 "$here/mut.py" M3b
