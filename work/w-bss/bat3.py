#!/usr/bin/env python3
"""Lane w-bss, battery 3: .data/.bss byte-level shape at the WORKLOAD's flags
(work/dc3-workload/flags.txt, which include /GR -- the CLI default set does not).

Every number this prints is transcribed from an obj produced by the real c2.dll
under wibo.  Nothing here computes an expected obj.

  WBSS_FLAGS=flags-w.txt  python3 bat3.py          # workload flags (/GR)
  WBSS_FLAGS=flags-O1.txt python3 bat3.py          # prereg flags (/GS-, no /GR)
"""
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from probe import compile_src
from coffdump import Obj, chdec

SEL = {0: '-', 1: 'NODUP', 2: 'ANY', 3: 'SAME_SIZE', 4: 'EXACT', 5: 'ASSOC', 6: 'LARGEST'}
SHELL = ('.drectve', '.debug$S')


def xbld(o, s):
    """Distinguish the two .XBLD$W watermarks by their first two raw bytes."""
    if s['name'] == '.XBLD$W':
        d = o.secdata(s)
        return '.XBLD$W(%s)' % (d[:2].decode('latin1') if d else '?')
    return s['name']


def aux_of(o, s):
    for sy in o.syms:
        if sy['sec'] == s['idx'] and sy['naux'] and sy['name'] == s['name']:
            return struct.unpack_from('<IHHIHB', sy['aux'][0], 0)   # len,nrel,nln,cks,num,sel
    return None


def show(tag, src, want=('.data', '.bss', '.tls', '.rdata'), raw=True, order_only=False):
    o = Obj(open(compile_src(src, tag), 'rb').read())
    print("### %s   nsec=%d nsym=%d" % (tag, o.nsec, o.nsym))
    print("    order: " + " ".join(xbld(o, s) for s in o.secs))
    if order_only:
        print()
        return o
    for s in o.secs:
        if not any(s['name'].startswith(p) for p in want):
            continue
        a = aux_of(o, s)
        cks = a[3] if a else 0
        sel = a[5] if a else 0
        print("    [%2d] %-9s size=0x%-5x ptr=0x%-5x vsz=0x%-3x nrel=%-2d ch=0x%08x %s"
              % (s['idx'], s['name'], s['size'], s['ptr'], s['vsz'], s['nrel'], s['ch'],
                 chdec(s['ch'])))
        if a:
            print("         aux: Length=0x%x NumberOfRelocations=%d NumberOfLinenumbers=%d "
                  "CheckSum=0x%08x Number=%d Selection=%s"
                  % (a[0], a[1], a[2], cks, a[4], SEL.get(sel, sel)))
        d = o.secdata(s)
        if raw and d and len(d) <= 64:
            print("         raw: " + d.hex(' '))
        for sy in o.syms:
            if sy['sec'] == s['idx'] and sy['naux'] == 0:
                print("         sym %-38s Value=0x%-5x Type=0x%04x SC=%d"
                      % (sy['name'][:38], sy['val'], sy['typ'], sy['sc']))
        for k in range(s['nrel']):
            va, sym, ty = struct.unpack_from('<IIH', o.data, s['ptrrel'] + k * 10)
            print("         rel off=0x%-4x type=0x%04x -> %s" % (va, ty, o.symname(sym)))
    for sy in o.syms:
        if sy['sec'] == 0:
            print("    UNDEF %-38s SC=%d Type=0x%04x" % (sy['name'][:38], sy['sc'], sy['typ']))
    print()
    return o


if __name__ == "__main__":
    print("FLAGS FILE: %s" % os.environ.get("WBSS_FLAGS", "flags-w.txt"))
    print(open(os.environ.get("WBSS_FLAGS", "flags-w.txt")).read())

    print("=" * 78)
    print("A. SECTION ORDER vs TU SHAPE")
    print("=" * 78)
    STR = 'const char* p = "zz";\n'
    for tag, src in [
        ('a_empty',   'void f();\n'),
        ('a_fn',      'int f(int x){return x+1;}\n'),
        ('a_bss',     'char b1;\n'),
        ('a_data',    'char d1=1;\n'),
        ('a_both',    'char b1;\nchar d1=1;\n'),
        ('a_dataonly2', 'char d1=1;\nint d2=2;\n'),
        ('a_str',     STR),
        ('a_strbss',  STR + 'char b1;\n'),
        ('a_constr',  'extern const int ce=9;\nint g(){return ce;}\n'),
        ('a_tls',     '__declspec(thread) int t1;\n'),
        ('a_tlsdata', '__declspec(thread) int t1=4;\n'),
        ('a_fnbss',   'char b1;\nint f(int x){return x+1;}\n'),
        ('a_bssfn',   'int f(int x){return x+1;}\nchar b1;\n'),
        ('a_dyn',     'struct L{L(int);};\nstatic L s(1);\n'),
        ('a_dynbss',  'struct L{L(int);};\nstatic L s(1);\nchar b1;\n'),
        ('a_dyndata', 'struct L{L(int);};\nstatic L s(1);\nchar d1=1;\n'),
        ('a_all',     'struct L{L(int);};\nchar b1;\nchar d1=1;\nconst char* p="zz";\n'
                      'extern const int ce=9;\nstatic L s(1);\nint f(int x){return x+1;}\n'),
    ]:
        show(tag, src, order_only=True)

    print("=" * 78)
    print("B. .bss AND .data HEADERS / ALIGNMENT GRID")
    print("=" * 78)
    show('b_bss_align', """
char a1; short a2; int a4; double a8; char a16[16]; char a64[64]; char a256[256];
__declspec(align(32)) char a32;
""")
    show('b_bss_1', 'char a1;\n')
    show('b_bss_2', 'short a2;\n')
    show('b_bss_4', 'int a4;\n')
    show('b_bss_8', 'double a8;\n')
    show('b_bss_c3', 'char a3[3];\n')
    show('b_bss_c5', 'char a5[5];\n')
    show('b_bss_c63', 'char a63[63];\n')
    show('b_bss_c64', 'char a64[64];\n')
    show('b_bss_c65', 'char a65[65];\n')
    show('b_data_init', """
char d1 = 1; short d2 = 2; int d4 = 4; double d8 = 8.0;
const char* dp = "hi";
char* dq = &d1;
int arr[4] = {1,2,3,4};
""")
    show('b_data_1', 'char d1=1;\n')
    show('b_data_4', 'int d4=4;\n')
    show('b_data_8', 'double d8=8.0;\n')
    show('b_data_c64', 'char d[64]={1};\n')
    show('b_data_align32', '__declspec(align(32)) int dq=1;\n')

    print("=" * 78)
    print("C. LINKAGE / QUALIFIER AXES")
    print("=" * 78)
    show('c_static_bss', 'static char s1;\nchar* f(){return &s1;}\n')
    show('c_static_data', 'static char s1=7;\nchar* f(){return &s1;}\n')
    show('c_extern_only', 'extern int eonly;\nint f(){return eonly;}\n')
    show('c_extern_decl_def', 'extern int ed;\nint ed=3;\nint f(){return ed;}\n')
    show('c_const', 'const int ci=7;\nextern const int ce=9;\nconst char cs[4]="abc";\n'
                    'int use(){return ci+ce+cs[0];}\n')
    show('c_const_unused', 'const int cu=7;\n')
    show('c_const_extern_unused', 'extern const int ceu=7;\n')
    show('c_selectany', '__declspec(selectany) int sa=5;\n__declspec(selectany) int sb;\n')
    show('c_selectany_used', '__declspec(selectany) int sa=5;\nint f(){return sa;}\n')
    show('c_classstatic', 'struct S{static int m; static const int k=3;};\nint S::m;\n'
                          'int f(){return S::m+S::k;}\n')
    show('c_classstatic_init', 'struct S{static int m;};\nint S::m=9;\nint f(){return S::m;}\n')
    show('c_volatile', 'volatile int v1;\nvolatile int v2=3;\n')
    show('c_zeroinit', 'int z1=0;\nint z2;\nint z3={0};\n')
    show('c_tls', '__declspec(thread) int t1;\n__declspec(thread) int t2=4;\n')
    show('c_hdr_shared', '#include "hdr_w.h"\nint f(){return hx+hy;}\n')
    show('c_array_partial', 'int ap[8]={1,2};\n')
    show('c_bigzero', 'int bz[1024];\n')

    print("=" * 78)
    print("D. RTTI / EH / VTABLE -- what actually lands in .data (contents, not names)")
    print("=" * 78)
    show('d_rtti_dyncast', 'struct B{virtual ~B();};\nstruct D:B{virtual ~D();};\n'
                           'B::~B(){}\nD::~D(){}\nB* cast(B*p){return dynamic_cast<D*>(p);}\n')
    show('d_eh_throw', 'void f(){ throw 1; }\n')
    show('d_eh_catch', 'struct T{~T();};\nvoid f(){T a; try{throw 1;}catch(int){}}\n')
    show('d_vt', 'struct V{virtual void f(); virtual ~V();};\nvoid V::f(){}\nV::~V(){}\n'
                 'V* mk(){return new V();}\n')
    show('d_typeid', '#include <typeinfo>\nstruct B{virtual ~B();};\n'
                     'const char* n(B*p){return typeid(*p).name();}\n')
