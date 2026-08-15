#!/bin/sh
# w-fencea — mutants graded by `cargo test` rather than by an obj: the two that
# fire the CLOSURE of the admission set, which no obj can see because it is a
# statement about classes that do not exist yet.
set -eu
R="$(cd "$(dirname "$0")/../.." && pwd)"
name="$1"; file="$2"; expr="$3"; want="$4"
cp "$R/$file" "$R/$file.orig"
sed -i "$expr" "$R/$file"
if cmp -s "$R/$file" "$R/$file.orig"; then
    echo "MUTANT $name: NOT APPLIED"; rm -f "$R/$file.orig"; exit 2
fi
echo "=== MUTANT $name  ($file) ==="
out=$(cd "$R" && cargo test -p c2-core --release --lib codegen::labels 2>&1 || true)
if printf '%s' "$out" | grep -q "error\[\|error:"; then
    echo "  BUILD-RED"
elif printf '%s' "$out" | grep -q "$want ... FAILED"; then
    echo "  RED: $want"
    printf '%s' "$out" | grep -c "FAILED" | sed 's/^/  failing tests: /'
else
    echo "  GREEN — the guard did not fire"
fi
mv "$R/$file.orig" "$R/$file"
