#!/usr/bin/env python3
"""Grade axes1: predicted (frozen in PREDICTIONS.md at commit 3401ffb) vs observed.

Predictions transcribed here verbatim from PREDICTIONS.md. `ALT` records the
pre-registered alternative readings; an observed set matching an ALT grades
AMBIGUOUS (defect in §2's *statement*), never VIOLATION — per the prereg's
guard 1 and the coordinator's binding condition.
"""
import json, os

A = '?anchor@@YAHH@Z'
P = {
 # ---- A1 -------------------------------------------------------------
 ('a1c1_depth1_inline_ref', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c2_depth5_inline_ref', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c3_depth5_inline_unref', 'main.obj'): {A},
 ('a1c4_depth3_static_one_ref', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c5_depth4_extern_def_unref', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c6_diamond_two_depths', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c7_use_before_def_depth5', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a1c8_chain_across_depths', 'main.obj'):
     {'?leafcand@@YAHH@Z', '?midcand@@YAHH@Z', '?topcand@@YAHH@Z', A},
 # ---- A5 -------------------------------------------------------------
 ('a5c1_externC_inline_unref', 'main.obj'): {A},
 ('a5c2_externC_inline_ref', 'main.obj'): {'cand', A},
 ('a5c3_static_inline_ref_and_unref', 'main.obj'): {'?candR@@YAHH@Z', A},
 ('a5c4_extern_then_inline_unref', 'main.obj'): {A},
 ('a5c5_header_static_one_ref', 'main.obj'): {'?hcandR@@YAHH@Z', A},
 ('a5c6_header_linkage_matrix', 'main.obj'):
     {'?hiR@@YAHH@Z', '?hsiR@@YAHH@Z', 'hciR', A},
 ('a5c7_header_externC_def_unref', 'main.obj'): {'hc', A},
 ('a5c8_header_static_inline_addr_in_data', 'main.obj'): {'?hsi@@YAHH@Z', A},
 ('a5c9_externC_static_inline', 'main.obj'): {'candR', A},
 # ---- A6 -------------------------------------------------------------
 ('a6c1_shared_inline_separate_invocations', 'tu1.obj'): {'?ca@@YAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c1_shared_inline_separate_invocations', 'tu2.obj'): {'?cb@@YAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c2_shared_inline_one_invocation', 'tu1.obj'): {'?ca@@YAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c2_shared_inline_one_invocation', 'tu2.obj'): {'?cb@@YAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c3_shared_inline_one_invocation_reversed', 'tu1.obj'): {'?ca@@YAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c3_shared_inline_one_invocation_reversed', 'tu2.obj'): {'?cb@@YAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c4_shared_static_one_tu_refs', 'tu1.obj'): {'?sa@@YAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c4_shared_static_one_tu_refs', 'tu2.obj'): {'?anchor2@@YAHH@Z'},
 ('a6c5_shared_vtable_one_tu_constructs', 'tu1.obj'):
     {'??0C@@QAA@XZ', '??1C@@UAA@XZ', '??_GC@@UAAPAXI@Z',
      '?v@C@@UAAHH@Z', '?w@C@@UAAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c5_shared_vtable_one_tu_constructs', 'tu2.obj'): {'?v@C@@UAAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c6_shared_extern_def_neither_refs', 'tu1.obj'): {'?hc@@YAHH@Z', '?anchor1@@YAHH@Z'},
 ('a6c6_shared_extern_def_neither_refs', 'tu2.obj'): {'?hc@@YAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c7_three_tus_middle_refs', 'tu1.obj'): {'?anchor1@@YAHH@Z'},
 ('a6c7_three_tus_middle_refs', 'tu2.obj'): {'?cb@@YAHH@Z', '?anchor2@@YAHH@Z'},
 ('a6c7_three_tus_middle_refs', 'tu3.obj'): {'?anchor3@@YAHH@Z'},
 ('a6c8_shared_dyninit_per_tu', 'tu1.obj'):
     {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', '?anchor1@@YAHH@Z'},
 ('a6c8_shared_dyninit_per_tu', 'tu2.obj'):
     {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', '?anchor2@@YAHH@Z'},
 # ---- A7 -------------------------------------------------------------
 ('a7c1_linker_include_static', 'main.obj'): {A},
 ('a7c2_linker_include_inline', 'main.obj'): {A},
 ('a7c3_comment_lib_inert', 'main.obj'): {A},
 ('a7c4_initseg_compiler', 'main.obj'): {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', A},
 ('a7c5_initseg_baseline_nopragma', 'main.obj'): {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', A},
 ('a7c6_initseg_lib', 'main.obj'): {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', A},
 ('a7c7_initseg_named_section', 'main.obj'): {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', A},
 ('a7c8_codeseg_static_unref', 'main.obj'): {A},
 ('a7c9_section_allocate_addrtake', 'main.obj'): {'?cand@@YAHH@Z', A},
 ('a7c10_initseg_internal_datum', 'main.obj'): {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', A},
 # ---- A8 -------------------------------------------------------------
 ('a8c1_yc_no_refs', 'pchgen.obj'): {'?anchorg@@YAHH@Z'},
 ('a8c2_yu_refs_ia_sa', 'pchgen.obj'): {'?anchorg@@YAHH@Z'},
 ('a8c2_yu_refs_ia_sa', 'user.obj'): {'?ia@@YAHH@Z', '?sa@@YAHH@Z', '?anchoru@@YAHH@Z'},
 ('a8c3_yu_no_refs', 'pchgen.obj'): {'?anchorg@@YAHH@Z'},
 ('a8c3_yu_no_refs', 'user.obj'): {'?anchoru@@YAHH@Z'},
 ('a8c4_nopch_control', 'user.obj'): {'?ia@@YAHH@Z', '?sa@@YAHH@Z', '?anchoru@@YAHH@Z'},
 ('a8c4_nopch_control', 'user2.obj'): {'?anchoru2@@YAHH@Z'},
 ('a8c5_extern_def_in_pch', 'pchgen.obj'): {'?ea@@YAHH@Z', '?anchorg@@YAHH@Z'},
 ('a8c5_extern_def_in_pch', 'user.obj'): {'?ea@@YAHH@Z', '?anchoru@@YAHH@Z'},
 ('a8c6_yc_refs_yu_does_not', 'pchgen.obj'):
     {'?ia@@YAHH@Z', '?sa@@YAHH@Z', '?anchorg@@YAHH@Z'},
 ('a8c6_yc_refs_yu_does_not', 'user.obj'): {'?anchoru@@YAHH@Z'},
 ('a8c7_pch_vtable', 'pchgen.obj'): {'?anchorg@@YAHH@Z'},
 ('a8c7_pch_vtable', 'user.obj'):
     {'??0C@@QAA@XZ', '??1C@@UAA@XZ', '??_GC@@UAAPAXI@Z',
      '?v@C@@UAAHH@Z', '?w@C@@UAAHH@Z', '?anchoru@@YAHH@Z'},
 ('a8c8_pch_dyninit', 'pchgen.obj'): {'?anchorg@@YAHH@Z'},
 ('a8c8_pch_dyninit', 'user.obj'):
     {'?mk@@YAHH@Z', '??__Eg_v@@YAXXZ', '?anchoru@@YAHH@Z'},
}

# pre-registered alternative readings (PREDICTIONS.md, "two derivation conventions")
ALT = {
 ('a5c1_externC_inline_unref', 'main.obj'): [{'cand', A}],
 ('a5c4_extern_then_inline_unref', 'main.obj'):
     [{'?cand@@YAHH@Z', A}, {'?cand2@@YAHH@Z', A}, {'?cand@@YAHH@Z', '?cand2@@YAHH@Z', A}],
}
# decoration-only variance accepted (PREDICTIONS.md, a5c9)
DEC = {('a5c9_externC_static_inline', 'main.obj'): [{'?candR@@YAHH@Z', A}]}

ROOT = os.path.dirname(os.path.abspath(__file__))
obs = {}
for x in json.load(open(os.path.join(ROOT, 'results.json'))):
    if x['listing']:
        continue
    for o, s in x['objs'].items():
        obs[(x['cell'], o)] = set(s['code_leaders']) if s else None

rows, counts = [], {}
for k in sorted(P, key=lambda k: (k[0][:4], k[0], k[1])):
    pred, got = P[k], obs.get(k)
    if got is None:
        v = 'INSTRUMENT-FAIL'
    elif got == pred or got in DEC.get(k, []):
        v = 'MATCH'
    elif got in ALT.get(k, []):
        v = 'AMBIGUOUS'
    else:
        v = 'VIOLATION'
    counts[v] = counts.get(v, 0) + 1
    rows.append((k[0], k[1], sorted(pred), sorted(got) if got else None, v,
                 sorted(got - pred) if got else [], sorted(pred - got) if got else []))

axis_of = lambda c: c[:2].upper()
per_axis = {}
for c, o, pr, ob, v, extra, missing in rows:
    d = per_axis.setdefault(axis_of(c), {})
    d[v] = d.get(v, 0) + 1
print('objs graded:', len(rows))
print('overall:', counts)
for ax in sorted(per_axis):
    print(f'  {ax}: {per_axis[ax]}')
print()
for c, o, pr, ob, v, extra, missing in rows:
    if v != 'MATCH':
        print(f'--- {v}  {c} / {o}')
        print(f'    predicted: {pr}')
        print(f'    observed : {ob}')
        if extra:
            print(f'    EXTRA (emitted, not predicted): {extra}')
        if missing:
            print(f'    MISSING (predicted, not emitted): {missing}')
json.dump([dict(cell=c, obj=o, predicted=pr, observed=ob, verdict=v,
                extra=e, missing=m) for c, o, pr, ob, v, e, m in rows],
          open(os.path.join(ROOT, 'grades.json'), 'w'), indent=1)
