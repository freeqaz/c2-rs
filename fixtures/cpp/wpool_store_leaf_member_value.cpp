// **Positive — lane `w-pool` (#2562).** THE CONTROL for the two `_neg` cells
// beside it, and the cell that says where `src/system/utl/Pool.cpp`'s distance
// is NOT.
//
// `Pool.cpp` is one of the **three** TUs in the 878-TU workload whose entire
// `decode_causes` set is `{body-out-of-class}` (#2506) — the reader is the only
// gate that fires. Its three functions total **132 bytes**, and its reference
// obj carries **zero relocations**, no `.pdata`, no `$M`/`$T` label symbol, no
// `_fltused` and no external beyond the two `__C*_11886` watermarks. There is
// no whole-obj obligation standing behind the reader here (`CEILING.md` §11's
// NC-1 and NC-2 are both empty on this TU), which is what makes its cells worth
// cutting at all.
//
// `?Free@Pool@@QAAXPAX@Z` is six words:
//
//     2b040000  cmplwi cr6,r4,0        the null guard
//     4d9a0020  bclr   12,26           …folded to a conditional RETURN (band 2)
//     81630000  lwz    r11,0(r3)       *(void**)v = p->mFree
//     91640000  stw    r11,0(r4)
//     90830000  stw    r4,0(r3)        p->mFree = (char*)v
//     4e800020  blr
//
// **This file is its third and fourth words and nothing else**, and it grades
// `Port=Match`, byte-exact. So the port already emits a store whose *value* is a
// member load, through a cast base, exactly as `Pool::Free` needs it — and
// `store-leaf` is the production that does it.
//
// That is what fixes the two `_neg` cells' fences in place. Without this control
// a reader takes `wpool_store_run_member_value_neg.cpp`'s refusal for *"the port
// cannot store a loaded value"*, which is false: it can, **once**. What it
// cannot do is put one inside a **run** — and one more store, with nothing else
// changed, is the whole of that cell.

struct WPoolP {
    char *mFree;
};

void wpool_store_leaf(WPoolP *p, void *v) { *(void **)v = p->mFree; }
