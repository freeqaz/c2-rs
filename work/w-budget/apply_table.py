#!/usr/bin/env python3
"""apply_table.py -- lane `w-budget`'s one edit to `work/w-inlmetric/CLAUSES.tsv`.

Committed rather than typed into a shell, for `#3451`'s reason one artifact
over: a table edit that exists only as a transcript cannot be re-checked, and
this one moves two `state` cells, one `blocker` cell and adds a column. Run
once; the result is the tracked table.

    python3 work/w-budget/apply_table.py
"""
import os

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
P = os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')

raw = open(P, encoding='utf-8').read().split('\n')
hi = [i for i, l in enumerate(raw) if l.startswith('id\t')][0]
cols = raw[hi].split('\t')
if 'blocker2' in cols:
    raise SystemExit('already applied')

rows, order = {}, []
for i in range(hi + 1, len(raw)):
    if not raw[i].strip():
        continue
    v = raw[i].split('\t')
    assert len(v) == len(cols), (i, len(v))
    d = dict(zip(cols, v))
    rows[d['id']] = d
    order.append(d['id'])


def setrow(rid, **kw):
    for k, v in kw.items():
        assert k in cols, k
        rows[rid][k] = v


# ---- C2: ADOPTED -----------------------------------------------------------
setrow('C2',
       clause='caller instruction count seeded: DAT_10c3f5cc = WORD [[fn]+0x50]',
       state='R-derived',
       witness='crates/c2-core/src/splice.rs:at_pass_entry_seeded',
       note=('ADOPTED by w-budget. The producing field is the .gl SIZE the port already decoded and DISCARDED '
             '(C24 note); it now seeds Expansion::growth_total and BudgetModel::seed. TWO corrections to the '
             'clause text, both from w-instrcount SS1: the load is [[fn]+0x50], ONE INDIRECTION (0x10b626f5 '
             'loads the SYMBOL, 0x10b626f7 the WORD), and the (ushort) was the movzx and nothing else -- the '
             'field is 16 bits at rest'),
       read='R1',
       readcite='docs/whitebox/WB_INSTRCOUNT_FINDINGS.md#0x10b626f7',
       blocker='none',
       egloss=('(F7 -- whose null WB_INSTRCOUNT SS5.1 re-measures as a property of the GRID: the axis moved '
               'B 1,000 to 9,846, not to 2,820)'),
       cites='crates/c2-core/src/splice.rs')

# ---- C16: ADOPTED ----------------------------------------------------------
setrow('C16',
       state='R-derived',
       witness='crates/c2-core/src/splice.rs:INLINE_GROWTH_TOTAL_MAX',
       note=('ADOPTED by w-budget as INLINE_GROWTH_TOTAL_MAX + declines_at_growth_total, a SETTABLE field of '
             'BudgetModel and not a baked 35000 -- a different immediate at a different site from the seed '
             "clamp's identical 35000. Byte-neutral ON THIS CORPUS rather than by construction: the largest "
             'measured total is 5,778 of 35,000 (WB_INSTRCOUNT SS5.2). c2 DOES fire it on the first site for a '
             'caller whose .gl SIZE byte is 0x81..0xff (sign-extended to 65,409..65,535); the port cannot be '
             'handed one, because the .gl reader refuses that encoding whole-file'),
       read='R1',
       blocker='none',
       cites='crates/c2-core/src/splice.rs')

# ---- C17: ADOPTED, and it was registered as conditional --------------------
# PREREG SS1 B6: C17 puts a NEW REFUSAL on a production path and a refusal that
# fires changes an emit, so it was registered as adoptable only if measured not
# to fire. Measured on both instruments -- gate identity diff 0 lines over 21
# rows, and the 878-TU scan pair IDENTICAL over 566 gap-metric keys with the
# workload stamp held. It does not fire.
setrow('C17',
       state='R-derived',
       witness='crates/c2-core/src/splice.rs:declines_unaffordable',
       note=('ADOPTED by w-budget as declines_unaffordable, in c2\'s own order (C16 at 0x10b60a63 first). '
             'WB_INSTRCOUNT SS7 recorded C17 as "blocker removed, still not adoptable" because [ebp+0x10] is '
             'threaded through c2\'s DRIVER; the port\'s chain walk IS that threading (Expansion steps down the '
             'chain) and on the set the port admits the two coincide EXACTLY -- S2 pins one call site per link, '
             'so c2 expands precisely the port\'s chain. Off that set the port has already refused. What is '
             'still absent is c2\'s FAN-OUT, which is blocker2 and is C4\'s driver'),
       read='R1',
       blocker='none',
       egloss=('(F7 -- and the first-site theorem, WB_INSTRCOUNT SS5.2: B >= 1000 for every caller, so an '
               'undrained budget cannot decline a callee under 1000)'),
       cites='crates/c2-core/src/splice.rs')

# ---- C24: the citation footprint moved; the state did not ------------------
setrow('C24',
       note=(rows['C24']['note'] +
             '. NO LONGER DISCARDED: w-budget added gl_function_instr_counts beside gl_function_attrs, off the '
             'same walk, so the VALUE now reaches the budget model. The row stays R-derived -- its counterpart '
             'was always the DECODE, and what changed is downstream of it'),
       cites='crates/c2-il/src/func/gl.rs')

# ---- C4: the one `blocker` cell this lane moves -----------------------------
setrow('C4',
       note=('TWO BLOCKERS, and the published cell named the smaller. The entry STATE is present, live and '
             'address-cited (Expansion::at_pass_entry, production caller in splice.rs); the budget=B ARGUMENT '
             'is CLOSED by w-budget; what remains is the DRIVER FUN_10b61ee1 itself -- a recursive walk with '
             'FAN-OUT, cited nowhere under crates/ -- and the site stream it walks. blocker re-pointed from '
             'no-instr-count (closed) to no-driver after reading the port, w-instrcount SS7 and w-clausegen '
             'RESULT SS3a'),
       blocker='no-driver')

# ---- the new column ---------------------------------------------------------
B2 = {
    'C1': 'no-pass', 'C2': '-', 'C3': '-', 'C4': 'no-instr-stream', 'C5': 'no-driver',
    'C6': 'emit-change', 'C7': 'value-refuted', 'C8': '-', 'C9': 'depends-on-C8',
    'C10': 'attr-hi-unread', 'C11': 'no-field-mapping', 'C12': 'attr-hi-unread', 'C13': '-',
    'C14': '-', 'C15': '-', 'C16': '-', 'C17': 'no-driver', 'C18': '-', 'C19': '-', 'C20': '-',
    'C21': '-', 'C22': '-', 'C23': '-', 'C24': '-',
}
cols.append('blocker2')
for rid in order:
    rows[rid]['blocker2'] = B2[rid]

LEGEND = """#
# ==========================================================================
# THE SECOND BLOCKER COLUMN, lane w-budget 2026-08-30, under
# work/w-budget/PREREG.md SS5. Board #3847, #3852. Working:
# work/w-budget/BLOCKER_AUDIT.md. This edit is scripted, not typed:
# work/w-budget/apply_table.py.
#
# #3847: `blocker` holds ONE cell per row and C4 provably needs two, and
# nobody had checked which OTHER rows are multiply blocked. Audited, all 12
# `absent` rows:
#
#   10 of the 12 carry a second blocker, and on FIVE (C1 C7 C9 C10 C12) the
#   SECOND is the BINDING one -- the published cell names the CHEAPER
#   obstacle. So #3816's `no-instr-count 4 / no-instr-stream 2 /
#   emit-change 5 / writer-unread 1` is not a partition of the WORK; it is a
#   partition of the first reason somebody wrote down.
#
# blocker2    the second obstacle, or `-` for "audited, none found". A row
#             that is not `absent` carries `-` because it was outside this
#             audit's population, NOT because it was checked and cleared.
#             Same vocabulary as `blocker`, plus six values it lacked:
#
#   no-pass          the port has no inline pass; the clause gates a thing
#                    that does not exist                                (C1)
#   no-driver        no recursive expansion driver with FAN-OUT; the port's
#                    walk is a chain pinned to one site per link  (C4 C5 C17)
#   value-refuted    read, and the value it would contribute is MEASURED not
#                    to reproduce c2's boundary: WB_INSTRCOUNT SS6's windows
#                    [261,267] static / [93,99] external contain no 0x10<<k
#                    and no single value fits both; #3732 bans 128     (C7)
#   depends-on-C8    blocked behind another row of this same table     (C9)
#   attr-hi-unread   the input is a [sym+0x4c] bit ABOVE the low byte and the
#                    port's .gl ATTR reader takes the low byte only, by its
#                    own doc: 0x2000 is bit 13, 0x80000 bit 19, 0x200 bit 9.
#                    **These two rows are marked R1 = read and derivable
#                    today, and they are not**                    (C10 C12)
#   no-field-mapping the tested field has no located counterpart in the IL
#                    container: [sym+0x20] is none of name/TYPE/offset/
#                    SRCPOS/SIZE/ATTR                                 (C11)
#
# no-driver is deliberately NOT no-instr-stream. C5/C6 want the STREAM; C4
# and C17 want the DRIVER that walks it. Collapsing them is what made the
# partition read as "one missing link, four rows".
#
# ONE `blocker` cell moved, C4's, and only after reading the port, both
# wave-20 reads and w-clausegen's reconciliation. A blocker cell is a
# verdict; no other one was touched.
#
# ADOPTIONS THIS WAVE, lane w-budget: C2 and C16 absent -> R-derived, so the
# split moves absent 12 -> 9 / R-derived 7 -> 10. C17 was registered as
# CONDITIONAL (PREREG SS1 B6) -- a new refusal on a production path may be
# adopted only if measured not to fire -- and it does not fire: gate identity
# diff 0 lines over 21 rows, 878-TU scan pair IDENTICAL over 566 keys with the
# stamp held. Its blocker2 stays `no-driver` because the row is adopted for a
# CHAIN and c2's driver has FAN-OUT. C20 is untouched: its `fitted` pin is the
# chain's closure, which the port's fixpoint already has.
# =========================================================================="""

out = raw[:hi]
out.append(LEGEND)
out.append('\t'.join(cols))
for rid in order:
    out.append('\t'.join(rows[rid][c] for c in cols))
out.append('')
open(P, 'w', encoding='utf-8').write('\n'.join(out))
print(f'rewrote {len(order)} rows, {len(cols)} columns')
