P = "docs/rungs/2026-08-06-w-fnbyte.md"
lines = open(P, encoding="utf-8").read().split("\n")
# 1-based lines 134..160 inclusive -> indices 133..159
assert lines[133].startswith("`differs` is printed as **witnesses**"), lines[133]
assert lines[159].strip() == "```", lines[159]
new = '''`differs` is printed as **witnesses**, not as a count: shape, port/ref/equal word
counts, the first disagreeing word in hex from **both sides**, and the mangled
symbol.

> **Two counts, and they answer different questions.** `fnbyte-differs` is
> **4,711** — one per `(TU, emitted function)` pair, the same unit as the
> denominator. `fnbyte-differs-witnesses` is **1,950** — *distinct mangled
> symbols*, because a template COMDAT is emitted into many TUs. The scan's
> printed signature table counts distinct symbols; the taxonomy below counts
> pairs. Quoting one against the other is how a header-inline population gets
> mistaken for a defect rate (board #222's shape). **601 of the 871 graded TUs
> carry at least one**, in **61 signatures**.

### 5.1 The taxonomy, from the scan's own witness keys

(`work/w-fnbyte/analyze.py`, over `differs` **pairs**;
`work/w-fnbyte/differ_taxonomy.txt` is its committed output.)

| family | n | shapes |
|---|---:|---|
| **A** — c2's whole body is a bare `blr`; the port emits a branch | **1,886** | `tail` |
| **B** — c2 emitted **fewer** words, no shared prefix | **1,484** | `seq` 1,361 · `framed` 123 |
| **C** — c2 emitted **more** words, no shared prefix | **1,157** | `tail` |
| **D** — a shared prefix, then a divergence | **184** | `seq` 180 · `tail` 4 |

**4,529 of 4,711 diverge at word 0.** These are not near misses; the port's body
is a different body.

The six largest signatures, **by pair**:

```
1516  tail  |w1/1/eq0 |first@0:port=48000000,ref=4e800020  e.g. ??1?$_STLP_alloc_proxy@PAHHV?$StlNodeAlloc@H@stlpmtx_std@@@stlpmtx_std@@QAA@XZ
 804  seq   |w12/2/eq0|first@0:port=7d8802a6,ref=90830000  e.g. ??0?$_List_iterator@HU?$_Nonconst_traits@H@stlpmtx_std@@@stlpmtx_std@@QAA@PAU_List_node_base@1@@Z
 542  seq   |w19/6/eq0|first@0:port=7d8802a6,ref=81630000  e.g. ??$?8PAUHamMoveKey@@@stlpmtx_std@@YA_NABV?$reverse_iterator@PAUHamMoveKey@@@0@0@Z
 370  tail  |w2/1/eq0 |first@0:port=38a00000,ref=4e800020  e.g. ??$_Destroy_Range@PAH@stlpmtx_std@@YAXPAH0@Z
 286  tail  |w3/7/eq0 |first@0:port=7c832378,ref=81640008  e.g. ?Release@Object@Hmx@@QAAXPAVObjRef@@@Z
 235  tail  |w1/3/eq0 |first@0:port=48000000,ref=81630000  e.g. ??C?$_List_iterator@...@stlpmtx_std@@QBAPAU?$pair@...@1@XZ
```

and the `framed` head, 83 of its 123 by distinct symbol:

```
framed|w9/3/eq0|first@0:port=7d8802a6,ref=81630004  e.g. ?back@?$vector@HV?$StlNodeAlloc@H@stlpmtx_std@@@stlpmtx_std@@QAAAAHXZ
```'''
out = lines[:133] + new.split("\n") + lines[160:]
open(P, "w", encoding="utf-8").write("\n".join(out))
print("rung §5 corrected")
