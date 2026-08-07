// GRID F2 CONTROL — the INDIRECT-LOAD value (`d->p = q->p;`), whose IL differs
// from the F2 address value by exactly the `30` byte. If the F2 production is
// tried in the wrong order, or if its `30` refusal is missing, this cell parses
// as an address-and-store (one `addi`) where c2 emits a load-and-store (`lwz` +
// `stw`) — a wrong-bytes emit rather than a gap.
struct BE { BE* mNext; BE* mPrev; };
struct S { BE* mFreeHead; BE* mUsedHead; };
void f2_ctrl_load(S* d, S* q) { d->mUsedHead = q->mFreeHead; }
