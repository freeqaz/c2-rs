#!/usr/bin/env python3
"""gen_table.py -- `P_INLINE.md` §6.1 is GENERATED from `CLAUSES.tsv`.

Lane `w-clausegen`, wave 20, board `#3817`-`#3823`, under
`work/w-clausegen/PREREG.md` §1.

# Why this exists

§6.1 and `CLAUSES.tsv` are the same instrument published twice, and they had
diverged in each of three consecutive hand re-syncs (`w-paramfill`,
`w-inlclause` + the coordinator, the wave-19 merge). `check_table.py` printed
**GREEN through every one of them** -- it grades the machine table and cannot
see the prose copy. Board `#3814`; named and deferred at `#3806` because the
lane that found it did not own the instrument.

Two copies of a fact is two chances to update one (`#3679`). This makes the page
a RENDERING and the TSV the only source, so the failure mode is not "re-sync
harder", it is **impossible**.

# What is generated, and what is not

Only the block between the two markers in `P_INLINE.md`:

    <!-- BEGIN GENERATED 6.1 ... -->   ...   <!-- END GENERATED 6.1 -->

which is the six-column table, the per-state split line and the exercised line.
Every word of §6's prose, §6.1's preamble and the blockquotes after it are
hand-written and untouched.

# Rendering rules -- deterministic, and that is the whole point

`CLAUSES.tsv` is an ASCII file by its own convention (`SS` for §, `x` for
multiply). The page is markdown. The transform between them is fixed here so
there is exactly one, and it is:

* `clause`   ASCII typography restored (` x ` -> ` x ` as multiply, `=>` ->
  arrow) and a single-pass backtick over four mechanical patterns: a
  `[reg+0xNN]` memory operand, a `DAT_`/`FUN_` symbol, a brace set containing a
  hex literal, and a bare hex literal. One pass, longest alternative first, so
  nothing double-wraps.
* `addr`     always `` `0x<addr>` ``.
* `state`    `absent` / `fitted` / `unexercisable` bolded verbatim; `R-derived`
  renders as the page's ``**`[R]`-derived**``. The page carried BOTH that
  spelling and a plain `**[R]-derived**` -- a drift this generator ends by
  having only one.
* `witness`  an `absent` or `unexercisable` row renders `-`. **This is not
  cosmetic.** Those rows cite a token that must stay ABSENT from `crates/`,
  and `check_table.py`'s ABSENCE screen greps the whole subtree including this
  page's own directory's siblings; a generator that spelled the token into a
  tracked file would redden the row it describes. Two rows were reddened that
  exact way by hand, and `clause_table.rs`'s doc comment records it. Other
  rows render `<basename>:<token>` plus `wgloss`.
* `exercised` `no` bolds; `not-separable` renders `not separable`; plus
  `egloss`.

`wgloss` and `egloss` exist so this generator drops NOTHING the hand-written
page published -- the parentheticals (`(F7)`, `(F8, 6 cells)`,
`` — `/O1` pins the bit ``) are page-only information, and generating without
them would be a silent narrowing (`#3748`).

# Usage

    gen_table.py [--check | --write] [PAGE]

`--check` (the default) diffs and exits non-zero on divergence, printing the
first differing lines. `PAGE` is positional so a **mutated copy** can be graded
-- which is how `clause_table.rs` watches this go RED without touching the
tracked file (`#3336`, `#3787`: a checker nobody has seen fail is decoration).

Read the verdict line, never the exit code.
"""
import csv, os, re, sys
from collections import Counter

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TSV = os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')
PAGE = os.path.join(REPO, 'docs/whitebox/ref/P_INLINE.md')

BEGIN = ('<!-- BEGIN GENERATED 6.1 -- work/w-inlmetric/gen_table.py --write; '
         'edit CLAUSES.tsv, never this block -->')
END = '<!-- END GENERATED 6.1 -->'

# One alternation, longest first, so a hex literal inside a memory operand or a
# brace set is consumed by the outer pattern and never wrapped twice.
CODE = re.compile(
    r'(\[[A-Za-z_][A-Za-z0-9_]*\+0x[0-9a-fA-F]+\]'      # [sym+0x50]
    r'|\{[^{}]*0x[0-9a-fA-F]+[^{}]*\}'                  # {0x400, 0x1000, ...}
    r'|\b(?:DAT|FUN|LAB)_[0-9a-fA-F]+\b'                # DAT_10c40ec4
    r'|__[A-Za-z][A-Za-z0-9_]*'                         # __forceinline
    r'|\*[a-z][A-Za-z0-9_]*'                            # *budget
    r'|\b0x[0-9a-fA-F]+\b)')                            # 0x28
# The `__ident` and `*ident` arms are NOT decoration. `__forceinline` and
# `*budget` are markdown emphasis openers; rendered bare they are one stray `__`
# or `*` elsewhere in the same cell away from swallowing the rest of the row.
# The TSV is ASCII and cannot know it is being put in a table -- so the renderer
# has to, and this is the only place that knowledge belongs.

STATE_MD = {
    'R-derived': '**`[R]`-derived**',
    'fitted': '**fitted**',
    'absent': '**absent**',
    'unexercisable': '**unexercisable**',
}
# Printed order is FIXED here, not by dict insertion, so a row moving in the TSV
# cannot silently reorder a published count line.
STATE_ORDER = ['R-derived', 'fitted', 'absent', 'unexercisable']
EX_ORDER = ['yes', 'no', 'not-separable', 'unexercisable']
EX_MD = {'yes': 'yes', 'no': '**no**', 'not-separable': 'not separable',
         'unexercisable': 'unexercisable'}
STATE_LABEL = {'R-derived': '`[R]`-derived', 'fitted': 'fitted',
               'absent': 'absent', 'unexercisable': 'unexercisable'}
EX_LABEL = {'yes': 'yes', 'no': 'no', 'not-separable': 'not separable',
            'unexercisable': 'unexercisable'}


def gloss(v):
    return '' if v in ('-', '') else v


def cell(s):
    """Escape the one character that would break a markdown table."""
    return s.replace('|', r'\|')


def clause_md(s):
    s = (s.replace(' x ', ' × ').replace('=>', '⇒')
          .replace(' >= ', ' ≥ ').replace(' <= ', ' ≤ '))
    return CODE.sub(r'`\1`', cell(s))


def render(rows):
    out = [BEGIN, '',
           '| # | clause | addr | state | witness | exercised by the workload |',
           '|---|---|---|---|---|---|']
    for r in rows:
        st = r['state']
        if st in ('absent', 'unexercisable'):
            wit = '—'
        else:
            path, _, tok = r['witness'].rpartition(':')
            wit = f'`{os.path.basename(path)}:{tok}`'
            if gloss(r['wgloss']):
                wit += ' ' + gloss(r['wgloss'])
        ex = EX_MD[r['exercised']]
        if gloss(r['egloss']):
            ex += ' ' + gloss(r['egloss'])
        out.append(f"| {r['id']} | {clause_md(r['clause'])} | `0x{r['addr']}` | "
                   f"{STATE_MD[st]} | {wit} | {ex} |")
    c, e, n = Counter(r['state'] for r in rows), Counter(r['exercised'] for r in rows), len(rows)
    out += ['',
            '**Per-state split: ' +
            ' · '.join(f'{STATE_LABEL[k]} {c[k]}' for k in STATE_ORDER) + '.**',
            '**Exercised: ' +
            ' · '.join(f'{EX_LABEL[k]} {e[k]}' for k in EX_ORDER) + '.**',
            '',
            f'> **Generated** from [`work/w-inlmetric/CLAUSES.tsv`]'
            f'(../../../work/w-inlmetric/CLAUSES.tsv) by '
            f'`work/w-inlmetric/gen_table.py --write`, over **{n}** rows. '
            f'`crates/c2-harness/tests/clause_table.rs` goes RED when this block and '
            f'the table diverge, so the three hand re-syncs of 2026-08-27..29 '
            f'(`#3814`) cannot recur. **Nothing between the markers is '
            f'hand-editable** -- edit `CLAUSES.tsv` and re-run.',
            END]
    return '\n'.join(out)


def rows():
    return list(csv.DictReader(
        [l for l in open(TSV, encoding='utf-8') if not l.startswith('#')], delimiter='\t'))


def splice(page_text, block):
    i, j = page_text.find(BEGIN), page_text.find(END)
    if i < 0 or j < 0:
        return None
    return page_text[:i] + block + page_text[j + len(END):]


def main(argv):
    mode = '--check'
    args = []
    for a in argv:
        if a in ('--check', '--write'):
            mode = a
        else:
            args.append(a)
    page = args[0] if args else PAGE
    rs = rows()
    block = render(rs)

    text = open(page, encoding='utf-8').read()
    i, j = text.find(BEGIN), text.find(END)
    print(f"table    : {os.path.relpath(TSV, REPO)}  ({len(rs)} rows)")
    print(f"page     : {os.path.relpath(page, REPO) if page.startswith(REPO) else page}")
    if i < 0 or j < 0:
        print(f"  MARKERS: MISSING -- expected {BEGIN!r} and {END!r}")
        print('\nTABLE-GEN: RED  (the page carries no generated block; '
              'nothing was compared)')
        return 1

    if mode == '--write':
        open(page, 'w', encoding='utf-8').write(splice(text, block))
        print(f"  wrote {len(block.splitlines())} generated lines")
        print('\nTABLE-GEN: WRITTEN')
        return 0

    have = text[i:j + len(END)]
    if have == block:
        print(f"  {len(block.splitlines())} generated lines match the table")
        print(f"\nTABLE-GEN: GREEN  (0 differing line(s) over {len(rs)} rows)")
        return 0

    a, b = have.split('\n'), block.split('\n')
    diffs = [(k, a[k] if k < len(a) else '<missing>', b[k] if k < len(b) else '<missing>')
             for k in range(max(len(a), len(b)))
             if (a[k] if k < len(a) else None) != (b[k] if k < len(b) else None)]
    for k, got, want in diffs[:8]:
        print(f"  FAIL line {k}: page has {got!r}")
        print(f"              table gives {want!r}")
    if len(diffs) > 8:
        print(f"  ... and {len(diffs) - 8} more")
    print(f"\nTABLE-GEN: RED  ({len(diffs)} differing line(s) over {len(rs)} rows)  "
          f"-- run `python3 work/w-inlmetric/gen_table.py --write`")
    return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
