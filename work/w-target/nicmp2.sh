#!/bin/sh
# nicmp2.sh — WHERE is `__declspec(noinline)` readable in the IL?
#
# Lane w-target. Two shapes, each as a matched pair that differs ONLY by the
# attribute, and each compiled under a filename of the SAME LENGTH — the `.gl`
# embeds the source path, so an unmatched pair shows a difference that is the
# path and not the attribute.
#
#   pair V — `void g(){ext();}`      the w04a shape (a chain intermediate)
#   pair I — `int g(int a){return a+1;}`  the w10 shape (a spliceable leaf)
#
# Usage:  work/w-target/nicmp2.sh
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
out=work/w-target/nipair
rm -rf "$out"
mkdir -p "$out"

mk() { printf '%s' "$2" > "$out/$1.cpp"; }

mk vaaaaaaaaaaaa 'void ext();
void g() { ext(); }
void f() { g(); }
'
mk vbbbbbbbbbbbb 'void ext();
__declspec(noinline) void g() { ext(); }
void f() { g(); }
'
mk iaaaaaaaaaaaa 'int gsink;
int g(int a) { return a + 1; }
int f(int a) { return g(a); }
'
mk ibbbbbbbbbbbb 'int gsink;
__declspec(noinline) int g(int a) { return a + 1; }
int f(int a) { return g(a); }
'

cap() {
    d="work/w-target/il/$1"
    rm -rf "$d"
    mkdir -p "$d"
    ./target/release/c2rs capture "$out/$1.cpp" --keep-il "$d" \
        --flags-file work/w-target/flags.txt --cwd . >/dev/null 2>&1
}

for n in vaaaaaaaaaaaa vbbbbbbbbbbbb iaaaaaaaaaaaa ibbbbbbbbbbbb; do cap "$n"; done

cmp_pair() {
    echo "== pair $1  ($2 vs $3)"
    for e in ex gl sy in db; do
        a=$(ls "work/w-target/il/$2"/*."$e")
        b=$(ls "work/w-target/il/$3"/*."$e")
        if cmp -s "$a" "$b"; then
            printf '   .%-3s BYTE-IDENTICAL  (%s B)\n' "$e" "$(stat -c%s "$a")"
        else
            printf '   .%-3s DIFFERS         sizes %s / %s\n' \
                "$e" "$(stat -c%s "$a")" "$(stat -c%s "$b")"
        fi
    done
}

cmp_pair V vaaaaaaaaaaaa vbbbbbbbbbbbb
cmp_pair I iaaaaaaaaaaaa ibbbbbbbbbbbb
