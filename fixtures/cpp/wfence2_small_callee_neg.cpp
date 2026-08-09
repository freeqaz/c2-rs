// w-fence2 NEGATIVE — the SIZE clause, at the composition seam.
// `Port=NotImplemented`, at every mode.
//
// **One field away from `wfence2_kept_local_callee.cpp`**, which is a whole-TU
// byte-exact match: the callee is small. Plain external, non-`inline`, `/O1`,
// lowerable — every one of the parser's conditions holds, so the TU is handed
// on — and its lowered body is far under `c2_core::comdat::INLINE_DECLINE_BYTES`
// (128), so `fenced_inlined_callee` refuses it one stage later.
//
// **The cell that says the two halves of the fence are BOTH live.** The parser
// exempts this TU and the obj is still not emitted; delete the seam and this
// file becomes a wrong obj rather than a refusal.
//
// `work/w-fence2/GRID-W.md`: over 7,552 intra-TU call edges in the 878-TU
// workload, read off the *reference* obj's own REL24 targets, a callee of 0–63
// emitted bytes was inlined **5,881 times and kept 0 times**. The seam does not
// have to be right about this one in particular — it refuses everything it
// cannot prove c2 KEPT, and 64–95 B is a measured MIXED band (146 kept against
// 570 inlined) that no rule may answer in either direction.
//
// Board rows #2470–#2478; `docs/rungs/2026-08-09-w-fence2.md`.

int wf2m_big(int a) { return a + 1; }

int wf2m_wrap(int a) { return wf2m_big(a + 1); }
