#!/usr/bin/env python3
"""axes1 cell generator — writes every designed cell for axes A1/A5/A6/A7/A8.

One auditable artifact: every cell's source text is a literal here, and every
cell's compile plan is a spec.json written beside it. Running this script is
idempotent and recreates the whole cell tree.

spec.json shape:
  {"invocations": [ {"args": [...extra cl args..., "src.cpp"],
                     "objs": ["tu1.obj", ...]}, ... ]}
Base flags (prepended by the runner): /O1 /Oi /EHsc /GS- /c
"""
import json, os

ROOT = os.path.dirname(os.path.abspath(__file__))
CELLS = os.path.join(ROOT, 'cells')

# ---------------------------------------------------------------- helpers
CELLDEFS = []


def cell(axis, name, files, invocations):
    CELLDEFS.append((axis, name, files, invocations))


def simple(src_name='main.cpp', extra=()):
    """one invocation, one source, obj named after the source"""
    obj = os.path.splitext(src_name)[0] + '.obj'
    return [{'args': list(extra) + [src_name], 'objs': [obj]}]


SINK = 'extern int sink(int);\n'
SEED = 'extern int seed();\n'
ANCHOR = 'int anchor(int x) { return sink(x) + 3; }\n'

# =====================================================================
# A1 — header inclusion depth
# =====================================================================

def chain(n, innermost_body, prefix='d'):
    """headers prefix1.h .. prefixN.h; prefixN includes prefixN-1 ... ; body in prefix1.h"""
    out = {}
    out[f'{prefix}1.h'] = f'#ifndef {prefix.upper()}1_H\n#define {prefix.upper()}1_H\n{innermost_body}#endif\n'
    for i in range(2, n + 1):
        out[f'{prefix}{i}.h'] = (f'#ifndef {prefix.upper()}{i}_H\n#define {prefix.upper()}{i}_H\n'
                                 f'#include "{prefix}{i-1}.h"\n#endif\n')
    return out


_INL_CAND = 'inline int cand(int x) { return x*3+1; }\n'

# a1c1 — inline at include depth 1, referenced
f = chain(1, _INL_CAND)
f['main.cpp'] = '#include "d1.h"\n' + SINK + 'int anchor(int x) { return cand(x) + sink(x); }\n'
cell('A1', 'a1c1_depth1_inline_ref', f, simple())

# a1c2 — the SAME inline definition at include depth 5, referenced
f = chain(5, _INL_CAND)
f['main.cpp'] = '#include "d5.h"\n' + SINK + 'int anchor(int x) { return cand(x) + sink(x); }\n'
cell('A1', 'a1c2_depth5_inline_ref', f, simple())

# a1c3 — inline at depth 5, NOT referenced
f = chain(5, _INL_CAND)
f['main.cpp'] = '#include "d5.h"\n' + SINK + ANCHOR
cell('A1', 'a1c3_depth5_inline_unref', f, simple())

# a1c4 — statics at depth 3: one referenced, one not
f = chain(3, 'static int cand(int x) { return x*3+1; }\n'
             'static int dead(int x) { return x-9; }\n')
f['main.cpp'] = '#include "d3.h"\n' + SINK + 'int anchor(int x) { return cand(x) + sink(x); }\n'
cell('A1', 'a1c4_depth3_static_one_ref', f, simple())

# a1c5 — external non-COMDAT out-of-line definition living at depth 4, unreferenced
f = chain(4, 'int cand(int x) { return x*3+1; }\n')
f['main.cpp'] = '#include "d4.h"\n' + SINK + ANCHOR
cell('A1', 'a1c5_depth4_extern_def_unref', f, simple())

# a1c6 — diamond: same leaf definition reachable at depth 2 and depth 3, guarded
f = {
    'leaf.h': '#ifndef LEAF_H\n#define LEAF_H\n' + _INL_CAND + '#endif\n',
    'left.h': '#ifndef LEFT_H\n#define LEFT_H\n#include "leaf.h"\n#endif\n',
    'mid.h': '#ifndef MID_H\n#define MID_H\n#include "left.h"\n#endif\n',
    'right.h': '#ifndef RIGHT_H\n#define RIGHT_H\n#include "leaf.h"\n#endif\n',
    'top.h': '#ifndef TOP_H\n#define TOP_H\n#include "mid.h"\n#include "right.h"\n#endif\n',
    'main.cpp': '#include "top.h"\n' + SINK + 'int anchor(int x) { return cand(x) + sink(x); }\n',
}
cell('A1', 'a1c6_diamond_two_depths', f, simple())

# a1c7 — the use precedes the definition; definition arrives at depth 5
f = chain(5, _INL_CAND)
f['main.cpp'] = ('inline int cand(int x);\n' + SINK +
                 'int anchor(int x) { return cand(x) + sink(x); }\n'
                 '#include "d5.h"\n')
cell('A1', 'a1c7_use_before_def_depth5', f, simple())

# a1c8 — transitive inline chain spanning three include depths
f = {
    'd1.h': '#ifndef D1_H\n#define D1_H\ninline int leafcand(int x) { return x*3+1; }\n#endif\n',
    'd2.h': '#ifndef D2_H\n#define D2_H\n#include "d1.h"\n#endif\n',
    'd3.h': '#ifndef D3_H\n#define D3_H\n#include "d2.h"\ninline int midcand(int x) { return leafcand(x) + 1; }\n#endif\n',
    'd4.h': '#ifndef D4_H\n#define D4_H\n#include "d3.h"\n#endif\n',
    'd5.h': '#ifndef D5_H\n#define D5_H\n#include "d4.h"\ninline int topcand(int x) { return midcand(x) * 2; }\n#endif\n',
    'main.cpp': '#include "d5.h"\n' + SINK + 'int anchor(int x) { return topcand(x) + sink(x); }\n',
}
cell('A1', 'a1c8_chain_across_depths', f, simple())

# =====================================================================
# A5 — static / inline / extern "C" / static inline crossings
# =====================================================================

# a5c1 — extern "C" inline, unreferenced
cell('A5', 'a5c1_externC_inline_unref',
     {'main.cpp': 'extern "C" inline int cand(int x) { return x*3+1; }\n' + SINK + ANCHOR},
     simple())

# a5c2 — extern "C" inline, referenced
cell('A5', 'a5c2_externC_inline_ref',
     {'main.cpp': 'extern "C" inline int cand(int x) { return x*3+1; }\n' + SINK +
                  'int anchor(int x) { return cand(x) + sink(x); }\n'},
     simple())

# a5c3 — static inline: one unreferenced, one referenced
cell('A5', 'a5c3_static_inline_ref_and_unref',
     {'main.cpp': 'static inline int candU(int x) { return x*3+1; }\n'
                  'static inline int candR(int x) { return x*5+2; }\n' + SINK +
                  'int anchor(int x) { return candR(x) + sink(x); }\n'},
     simple())

# a5c4 — inline reached through a prior extern declaration; and `extern inline`
cell('A5', 'a5c4_extern_then_inline_unref',
     {'main.cpp': 'extern int cand(int x);\n'
                  'inline int cand(int x) { return x*3+1; }\n'
                  'extern inline int cand2(int x) { return x*7+4; }\n' + SINK + ANCHOR},
     simple())

# a5c5 — statics DEFINED IN A HEADER, one referenced one not
cell('A5', 'a5c5_header_static_one_ref',
     {'hdr.h': '#ifndef HDR_H\n#define HDR_H\n'
               'static int hcandR(int x) { return x*3+1; }\n'
               'static int hcandU(int x) { return x-9; }\n#endif\n',
      'main.cpp': '#include "hdr.h"\n' + SINK +
                  'int anchor(int x) { return hcandR(x) + sink(x); }\n'},
     simple())

# a5c6 — six header-defined linkage classes, three referenced three not
cell('A5', 'a5c6_header_linkage_matrix',
     {'hdr.h': '#ifndef HDR_H\n#define HDR_H\n'
               'inline int hiR(int x) { return x*3+1; }\n'
               'inline int hiU(int x) { return x*3+2; }\n'
               'static inline int hsiR(int x) { return x*5+1; }\n'
               'static inline int hsiU(int x) { return x*5+2; }\n'
               'extern "C" inline int hciR(int x) { return x*7+1; }\n'
               'extern "C" inline int hciU(int x) { return x*7+2; }\n#endif\n',
      'main.cpp': '#include "hdr.h"\n' + SINK +
                  'int anchor(int x) { return hiR(x) + hsiR(x) + hciR(x) + sink(x); }\n'},
     simple())

# a5c7 — extern "C" NON-inline out-of-line definition in a header, unreferenced
cell('A5', 'a5c7_header_externC_def_unref',
     {'hdr.h': '#ifndef HDR_H\n#define HDR_H\n'
               'extern "C" int hc(int x) { return x*3+1; }\n#endif\n',
      'main.cpp': '#include "hdr.h"\n' + SINK + ANCHOR},
     simple())

# a5c8 — header static inline reached only through a kept data initializer
cell('A5', 'a5c8_header_static_inline_addr_in_data',
     {'hdr.h': '#ifndef HDR_H\n#define HDR_H\n'
               'static inline int hsi(int x) { return x*3+1; }\n#endif\n',
      'main.cpp': '#include "hdr.h"\n'
                  'int (*g_p)(int) = &hsi;\n' + SINK + ANCHOR},
     simple())

# a5c9 — extern "C" static inline crossing, one referenced one not
cell('A5', 'a5c9_externC_static_inline',
     {'main.cpp': 'extern "C" static inline int candR(int x) { return x*3+1; }\n'
                  'extern "C" static inline int candU(int x) { return x*3+2; }\n' + SINK +
                  'int anchor(int x) { return candR(x) + sink(x); }\n'},
     simple())

# =====================================================================
# A6 — multiple TUs sharing one header, per-TU differing references
# =====================================================================

_SH_INL = ('#ifndef SHARED_H\n#define SHARED_H\n'
           'inline int ca(int x) { return x*3+1; }\n'
           'inline int cb(int x) { return x*5+2; }\n'
           'inline int cc(int x) { return x*7+3; }\n#endif\n')

_TU1_A = '#include "shared.h"\n' + SINK + 'int anchor1(int x) { return ca(x) + sink(x); }\n'
_TU2_B = '#include "shared.h"\n' + SINK + 'int anchor2(int x) { return cb(x) + sink(x); }\n'

# a6c1 — two separate cl invocations (baseline for per-TU independence)
cell('A6', 'a6c1_shared_inline_separate_invocations',
     {'shared.h': _SH_INL, 'tu1.cpp': _TU1_A, 'tu2.cpp': _TU2_B},
     [{'args': ['tu1.cpp'], 'objs': ['tu1.obj']},
      {'args': ['tu2.cpp'], 'objs': ['tu2.obj']}])

# a6c2 — identical sources, ONE cl invocation compiling both TUs
cell('A6', 'a6c2_shared_inline_one_invocation',
     {'shared.h': _SH_INL, 'tu1.cpp': _TU1_A, 'tu2.cpp': _TU2_B},
     [{'args': ['tu1.cpp', 'tu2.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# a6c3 — same, command-line order reversed
cell('A6', 'a6c3_shared_inline_one_invocation_reversed',
     {'shared.h': _SH_INL, 'tu1.cpp': _TU1_A, 'tu2.cpp': _TU2_B},
     [{'args': ['tu2.cpp', 'tu1.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# a6c4 — shared header statics; TU1 references one, TU2 references none
cell('A6', 'a6c4_shared_static_one_tu_refs',
     {'shared.h': '#ifndef SHARED_H\n#define SHARED_H\n'
                  'static int sa(int x) { return x*3+1; }\n'
                  'static int sb(int x) { return x*5+2; }\n#endif\n',
      'tu1.cpp': '#include "shared.h"\n' + SINK + 'int anchor1(int x) { return sa(x) + sink(x); }\n',
      'tu2.cpp': '#include "shared.h"\n' + SINK + 'int anchor2(int x) { return sink(x) + 3; }\n'},
     [{'args': ['tu1.cpp', 'tu2.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# a6c5 — shared polymorphic class: TU1 constructs it, TU2 only virtual-calls through a pointer
cell('A6', 'a6c5_shared_vtable_one_tu_constructs',
     {'shared.h': '#ifndef SHARED_H\n#define SHARED_H\n'
                  'struct C {\n'
                  '  int f;\n'
                  '  C() : f(1) {}\n'
                  '  virtual ~C() {}\n'
                  '  virtual int v(int x) { return x + f; }\n'
                  '  virtual int w(int x) { return x - f; }\n'
                  '};\n#endif\n',
      'tu1.cpp': '#include "shared.h"\n' + SINK +
                 'int anchor1(int x) { C c; return c.v(x) + sink(x); }\n',
      'tu2.cpp': '#include "shared.h"\n' + SINK + 'extern C* pc;\n'
                 'int anchor2(int x) { return pc->v(x) + sink(x); }\n'},
     [{'args': ['tu1.cpp', 'tu2.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# a6c6 — header carries an external non-COMDAT definition; neither TU references it
cell('A6', 'a6c6_shared_extern_def_neither_refs',
     {'shared.h': '#ifndef SHARED_H\n#define SHARED_H\n'
                  'int hc(int x) { return x*3+1; }\n#endif\n',
      'tu1.cpp': '#include "shared.h"\n' + SINK + 'int anchor1(int x) { return sink(x) + 3; }\n',
      'tu2.cpp': '#include "shared.h"\n' + SINK + 'int anchor2(int x) { return sink(x) + 4; }\n'},
     [{'args': ['tu1.cpp', 'tu2.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# a6c7 — three TUs in one invocation, only the middle one references
cell('A6', 'a6c7_three_tus_middle_refs',
     {'shared.h': _SH_INL,
      'tu1.cpp': '#include "shared.h"\n' + SINK + 'int anchor1(int x) { return sink(x) + 3; }\n',
      'tu2.cpp': '#include "shared.h"\n' + SINK + 'int anchor2(int x) { return cb(x) + sink(x); }\n',
      'tu3.cpp': '#include "shared.h"\n' + SINK + 'int anchor3(int x) { return sink(x) + 5; }\n'},
     [{'args': ['tu1.cpp', 'tu2.cpp', 'tu3.cpp'], 'objs': ['tu1.obj', 'tu2.obj', 'tu3.obj']}])

# a6c8 — header-defined internal dynamic-init datum: an independent root in each TU
cell('A6', 'a6c8_shared_dyninit_per_tu',
     {'shared.h': '#ifndef SHARED_H\n#define SHARED_H\n' + SEED +
                  'inline int mk(int x) { return x*3+1; }\n'
                  'static int g_v = mk(seed());\n#endif\n',
      'tu1.cpp': '#include "shared.h"\n' + SINK + 'int anchor1(int x) { return sink(x) + g_v; }\n',
      'tu2.cpp': '#include "shared.h"\n' + SINK + 'int anchor2(int x) { return sink(x) + g_v; }\n'},
     [{'args': ['tu1.cpp', 'tu2.cpp'], 'objs': ['tu1.obj', 'tu2.obj']}])

# =====================================================================
# A7 — pragma-created roots
# =====================================================================

# a7c1 — #pragma comment(linker,"/include:") naming an unreferenced static
cell('A7', 'a7c1_linker_include_static',
     {'main.cpp': '#pragma comment(linker, "/include:?cand@@YAHH@Z")\n'
                  'static int cand(int x) { return x*3+1; }\n' + SINK + ANCHOR},
     simple())

# a7c2 — same, naming an unreferenced inline
cell('A7', 'a7c2_linker_include_inline',
     {'main.cpp': '#pragma comment(linker, "/include:?cand@@YAHH@Z")\n'
                  'inline int cand(int x) { return x*3+1; }\n' + SINK + ANCHOR},
     simple())

# a7c3 — inert pragmas (lib/exestr) beside an unreferenced static: control
cell('A7', 'a7c3_comment_lib_inert',
     {'main.cpp': '#pragma comment(lib, "foo")\n'
                  '#pragma comment(exestr, "axes1")\n'
                  'static int cand(int x) { return x*3+1; }\n' + SINK + ANCHOR},
     simple())

_DYNINIT_BODY = (SEED + 'static int mk(int x) { return x*3+1; }\n'
                 'int g_v = mk(seed());\n' + SINK + ANCHOR)

# a7c4 — init_seg(compiler)
cell('A7', 'a7c4_initseg_compiler',
     {'main.cpp': '#pragma init_seg(compiler)\n' + _DYNINIT_BODY}, simple())

# a7c5 — no pragma: the paired control for a7c4/a7c6/a7c7
cell('A7', 'a7c5_initseg_baseline_nopragma',
     {'main.cpp': _DYNINIT_BODY}, simple())

# a7c6 — init_seg(lib)
cell('A7', 'a7c6_initseg_lib',
     {'main.cpp': '#pragma init_seg(lib)\n' + _DYNINIT_BODY}, simple())

# a7c7 — init_seg with a user-named section
cell('A7', 'a7c7_initseg_named_section',
     {'main.cpp': '#pragma init_seg(".mycrt$a")\n' + _DYNINIT_BODY}, simple())

# a7c8 — #pragma code_seg over an unreferenced static
cell('A7', 'a7c8_codeseg_static_unref',
     {'main.cpp': '#pragma code_seg(".mytext")\n'
                  'static int cand(int x) { return x*3+1; }\n' + SINK + ANCHOR},
     simple())

# a7c9 — #pragma section + __declspec(allocate) datum whose initializer takes a static's address
cell('A7', 'a7c9_section_allocate_addrtake',
     {'main.cpp': '#pragma section(".mysec", read, write)\n'
                  'static int cand(int x) { return x*3+1; }\n'
                  '__declspec(allocate(".mysec")) int (*g_p)(int) = &cand;\n' + SINK + ANCHOR},
     simple())

# a7c10 — init_seg(compiler) over an INTERNAL dynamic-init datum
cell('A7', 'a7c10_initseg_internal_datum',
     {'main.cpp': '#pragma init_seg(compiler)\n' + SEED +
                  'static int mk(int x) { return x*3+1; }\n'
                  'static int g_v = mk(seed());\n' + SINK +
                  'int anchor(int x) { return sink(x) + g_v; }\n'},
     simple())

# =====================================================================
# A8 — PCH (/Yc, /Yu)
# =====================================================================

_PCHA = ('#ifndef PCHA_H\n#define PCHA_H\n'
         'inline int ia(int x) { return x*3+1; }\n'
         'inline int ib(int x) { return x*3+2; }\n'
         'static int sa(int x) { return x*5+1; }\n'
         'static int sb(int x) { return x*5+2; }\n#endif\n')

# a8c1 — the /Yc TU itself references nothing from the pch
cell('A8', 'a8c1_yc_no_refs',
     {'pcha.h': _PCHA,
      'pchgen.cpp': '#include "pcha.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n'},
     [{'args': ['/Ycpcha.h', '/Fpa8c1.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']}])

# a8c2 — /Yu TU references one inline + one static from the pch
cell('A8', 'a8c2_yu_refs_ia_sa',
     {'pcha.h': _PCHA,
      'pchgen.cpp': '#include "pcha.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n',
      'user.cpp': '#include "pcha.h"\n' + SINK +
                  'int anchoru(int x) { return ia(x) + sa(x) + sink(x); }\n'},
     [{'args': ['/Ycpcha.h', '/Fpa8c2.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupcha.h', '/Fpa8c2.pch', 'user.cpp'], 'objs': ['user.obj']}])

# a8c3 — /Yu TU references nothing from the pch
cell('A8', 'a8c3_yu_no_refs',
     {'pcha.h': _PCHA,
      'pchgen.cpp': '#include "pcha.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n',
      'user.cpp': '#include "pcha.h"\n' + SINK + 'int anchoru(int x) { return sink(x) + 3; }\n'},
     [{'args': ['/Ycpcha.h', '/Fpa8c3.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupcha.h', '/Fpa8c3.pch', 'user.cpp'], 'objs': ['user.obj']}])

# a8c4 — byte-identical user.cpp sources compiled WITHOUT pch: the paired control
cell('A8', 'a8c4_nopch_control',
     {'pcha.h': _PCHA,
      'user.cpp': '#include "pcha.h"\n' + SINK +
                  'int anchoru(int x) { return ia(x) + sa(x) + sink(x); }\n',
      'user2.cpp': '#include "pcha.h"\n' + SINK + 'int anchoru2(int x) { return sink(x) + 3; }\n'},
     [{'args': ['user.cpp', 'user2.cpp'], 'objs': ['user.obj', 'user2.obj']}])

# a8c5 — an external non-COMDAT definition living inside the pch header
cell('A8', 'a8c5_extern_def_in_pch',
     {'pchb.h': '#ifndef PCHB_H\n#define PCHB_H\n'
                'int ea(int x) { return x*3+1; }\n'
                'inline int ib(int x) { return x*3+2; }\n#endif\n',
      'pchgen.cpp': '#include "pchb.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n',
      'user.cpp': '#include "pchb.h"\n' + SINK + 'int anchoru(int x) { return sink(x) + 4; }\n'},
     [{'args': ['/Ycpchb.h', '/Fpa8c5.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupchb.h', '/Fpa8c5.pch', 'user.cpp'], 'objs': ['user.obj']}])

# a8c6 — the /Yc TU references ia; the /Yu TU references nothing (leakage-through-pch test)
cell('A8', 'a8c6_yc_refs_yu_does_not',
     {'pcha.h': _PCHA,
      'pchgen.cpp': '#include "pcha.h"\n' + SINK +
                    'int anchorg(int x) { return ia(x) + sa(x) + sink(x); }\n',
      'user.cpp': '#include "pcha.h"\n' + SINK + 'int anchoru(int x) { return sink(x) + 3; }\n'},
     [{'args': ['/Ycpcha.h', '/Fpa8c6.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupcha.h', '/Fpa8c6.pch', 'user.cpp'], 'objs': ['user.obj']}])

# a8c7 — polymorphic class defined inside the pch; /Yu TU constructs it
cell('A8', 'a8c7_pch_vtable',
     {'pchc.h': '#ifndef PCHC_H\n#define PCHC_H\n'
                'struct C {\n'
                '  int f;\n'
                '  C() : f(1) {}\n'
                '  virtual ~C() {}\n'
                '  virtual int v(int x) { return x + f; }\n'
                '  virtual int w(int x) { return x - f; }\n'
                '};\n#endif\n',
      'pchgen.cpp': '#include "pchc.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n',
      'user.cpp': '#include "pchc.h"\n' + SINK +
                  'int anchoru(int x) { C c; return c.v(x) + sink(x); }\n'},
     [{'args': ['/Ycpchc.h', '/Fpa8c7.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupchc.h', '/Fpa8c7.pch', 'user.cpp'], 'objs': ['user.obj']}])

# a8c8 — dynamic-init root in the /Yu TU over a pch-defined static
cell('A8', 'a8c8_pch_dyninit',
     {'pchd.h': '#ifndef PCHD_H\n#define PCHD_H\n' + SEED +
                'static int mk(int x) { return x*3+1; }\n'
                'static int nomk(int x) { return x*9+7; }\n#endif\n',
      'pchgen.cpp': '#include "pchd.h"\n' + SINK + 'int anchorg(int x) { return sink(x) + 3; }\n',
      'user.cpp': '#include "pchd.h"\n' + 'int g_v = mk(seed());\n' + SINK +
                  'int anchoru(int x) { return sink(x) + g_v; }\n'},
     [{'args': ['/Ycpchd.h', '/Fpa8c8.pch', 'pchgen.cpp'], 'objs': ['pchgen.obj']},
      {'args': ['/Yupchd.h', '/Fpa8c8.pch', 'user.cpp'], 'objs': ['user.obj']}])


# ---------------------------------------------------------------- write
def main():
    n = 0
    for axis, name, files, invocations in CELLDEFS:
        d = os.path.join(CELLS, axis, name)
        os.makedirs(d, exist_ok=True)
        for fn, txt in files.items():
            with open(os.path.join(d, fn), 'w') as fh:
                fh.write(txt)
        with open(os.path.join(d, 'spec.json'), 'w') as fh:
            json.dump({'axis': axis, 'cell': name, 'invocations': invocations}, fh, indent=1)
        n += 1
    per = {}
    for axis, name, _f, _i in CELLDEFS:
        per[axis] = per.get(axis, 0) + 1
    print('cells written:', n, per)


if __name__ == '__main__':
    main()
