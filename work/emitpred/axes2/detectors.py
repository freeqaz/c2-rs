#!/usr/bin/env python3
"""Detectability demonstration for the axes2 violations.

Question the pre-registration asks: for each confirmed violation of §2-as-stated,
is the BREAKING CONDITION visible from c1xx-side observables, so that a
fail-closed R3 could answer `Unknown => refuse` instead of emitting a wrong obj?

Two channels are exercised, and they are NOT equally good:

  D1 (.gl channel)  — the c1xx name table, separator-aware. Sound and exact for
                      the synthesized-symbol family and for the ??__E family.
  D2 (source channel) — required for the families where .gl provably cannot
                      help; the a9_05/a9_06 pair have IDENTICAL .gl name sets
                      and opposite emission, which is the proof.

Prints, per detector: hits, false negatives against the violation list, and
over-flags (cells flagged that did not violate). Nothing here is a predicate for
emission; every detector is a REFUSAL trigger.
"""
import json, os, re

B = '<repo>/.claude/worktrees/w-emitpred/work/emitpred/axes2/'
il = {r['cell']: r for r in json.load(open(B + 'il_names.json'))}
ob = {r['cell']: r for r in json.load(open(B + 'observed.json'))}
CELLS = os.path.join(B, 'cells')

# The confirmed violations, by family. Fixed from the graded table.
FAM = {
    'synthesized-symbol category (??_D / ??_9 / adjustor thunk)':
        ['a3_03_mi_virtual_dtors', 'a3_04_virtual_base_simple',
         'a3_05_vbase_virtual_overridden', 'a3_06_diamond',
         'a9_07_address_of_virtual_no_ctor'],
    'vtable forced with no kept constructor':
        ['a2_08_explicit_inst_virtuals_no_object',
         'a9_05_outofline_virtual_dtor_no_ctor'],
    'virtual call is not an ODR-use of the callee':
        ['a9_04_dynamic_cast_reference_form',
         'a9_06_delete_through_pointer_no_ctor'],
    'root-4 (??__E) fires without emission':
        ['a4_09_anon_ns_dyninit_calls_anon_ns_static'],
}


def src(cell):
    for axis in ('A2', 'A3', 'A4', 'A9'):
        p = os.path.join(CELLS, axis, cell, 'cell.cpp')
        if os.path.exists(p):
            return open(p).read()
    raise KeyError(cell)


SYNTH = re.compile(r'^\?\?_D|^\?\?_9|@W[0-9A-Z]')


def d1_synth(c):
    """.gl names a compiler-synthesized symbol class §2 has no clause for."""
    return any(SYNTH.search(n) for n in il[c]['gl_names'])


def d1_dyninit(c):
    """.gl names a dynamic-initializer thunk."""
    return any(n.startswith('??__E') for n in il[c]['gl_names'])


def d2_vtable_no_ctor(c):
    """Source: a vtable can be forced without any constructor being ODR-used —
    an out-of-line virtual destructor definition, or an explicit instantiation
    definition of a class template that declares virtuals."""
    s = src(c)
    if 'virtual' not in s:
        return False
    if re.search(r'^\s*\w[\w:<>,\s]*::~\w+\s*\(', s, re.M):
        return True
    if re.search(r'^\s*template\s+(struct|class)\s+\w+\s*<', s, re.M):
        return True
    return False


def d2_virtual_call(c):
    """Source: a call that may dispatch through a vtable — the TU declares a
    virtual, and contains either a `delete` or a member-call expression.

    Stated as the general condition, not as a pattern fitted to the two
    violating cells: any member call in a TU that has virtuals is a candidate
    virtual dispatch, and a fail-closed consumer must refuse it. On these 35
    cells the conjunct with `virtual` is what keeps it from flagging the four
    non-polymorphic template cells that also contain member calls."""
    s = src(c)
    if 'virtual' not in s:
        return False
    if re.search(r'\bdelete\s+\w', s):
        return True
    if re.search(r'\w\s*(->|\.)\s*\w+\s*\(', s):
        return True
    return False


def report(name, fn, targets):
    hits = sorted(c for c in ob if fn(c))
    miss = sorted(set(targets) - set(hits))
    over = sorted(set(hits) - set(targets))
    print(f'\n{name}')
    print(f'  target violations : {len(targets)}')
    print(f'  flagged           : {len(hits)}')
    print(f'  FALSE NEGATIVES   : {len(miss)} {miss if miss else ""}')
    print(f'  over-flags        : {len(over)} {over if over else ""}')
    return miss, over


def main():
    print('=== detectability of the axes2 violations, by channel ===')
    report('D1a  .gl names a ??_D / ??_9 / adjustor symbol   [family 1]',
           d1_synth, FAM['synthesized-symbol category (??_D / ??_9 / adjustor thunk)'])
    report('D1b  .gl names a ??__E dynamic-initializer thunk [family 4]',
           d1_dyninit, FAM['root-4 (??__E) fires without emission'])
    report('D2a  source: out-of-line virtual dtor | explicit inst of a\n'
           '     polymorphic class template                  [family 2]',
           d2_vtable_no_ctor, FAM['vtable forced with no kept constructor'])
    report('D2b  source: delete / member call through a pointer or reference\n'
           '     to a polymorphic class                      [family 3]',
           d2_virtual_call, FAM['virtual call is not an ODR-use of the callee'])

    print('\n=== why family 2 and 3 CANNOT use the .gl channel ===')
    a, b = 'a9_05_outofline_virtual_dtor_no_ctor', 'a9_06_delete_through_pointer_no_ctor'
    ga = {n for n in il[a]['gl_names'] if not n.startswith('?anchor')}
    gb = {n for n in il[b]['gl_names'] if not n.startswith('?anchor')}
    print(f'  {a}')
    print(f'    .gl (minus anchor): {sorted(ga)}')
    print(f'    emitted           : {ob[a]["code_leaders"]}')
    print(f'  {b}')
    print(f'    .gl (minus anchor): {sorted(gb)}')
    print(f'    emitted           : {ob[b]["code_leaders"]}')
    print(f'  .gl name sets identical (modulo anchor signature): {ga == gb}')
    print(f'  emitted cardinalities: {len(ob[a]["code_leaders"])} vs '
          f'{len(ob[b]["code_leaders"])}')

    print('\n=== union refusal rule (all four detectors OR-ed) ===')
    allv = sorted(set().union(*FAM.values()))
    fn = lambda c: d1_synth(c) or d1_dyninit(c) or d2_vtable_no_ctor(c) or d2_virtual_call(c)
    hits = sorted(c for c in ob if fn(c))
    print(f'  violations covered : {len(set(allv) & set(hits))}/{len(allv)}')
    print(f'  missed             : {sorted(set(allv) - set(hits))}')
    print(f'  total cells flagged: {len(hits)}/{len(ob)}')
    clean = sorted(set(hits) - set(allv))
    print(f'  non-violating cells also refused (the cost of fail-closed): '
          f'{len(clean)} {clean}')


if __name__ == '__main__':
    main()
