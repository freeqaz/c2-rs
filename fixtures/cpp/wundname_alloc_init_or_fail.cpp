// W-UNDNAME — the guarded allocation with a shared error store, the class
// `src/xdk/LIBCMT/undname.cpp`'s `?append@DName@@QAAXPAVDNameNode@@@Z` is the
// workload instance of.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/1 in class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself: it is the first obj this
// port emits whose undefined externals INTERLEAVE — `data · callee · data` down
// `.text` — and therefore the only graded cell for board **#1720**, the merged
// symbol-order rule GRID A measured and w-extdata could not ship. A writer
// emitting callees and then data symbols puts `?getMemory` in the wrong slot
// here, and every relocation still resolves.

struct DNameNode;

struct HeapManager {
    void *getMemory(int size, int flags);
};

extern HeapManager gHeapManager;
extern void *pairNode_vtable;

struct pairNode {
    void **vtable;
    const DNameNode *left;
    DNameNode *right;
    int refcount;
};

struct DName {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n);
};

void DName::append(DNameNode *n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right = n;
            p->refcount = -1;
            p->vtable = &pairNode_vtable;
            p->left = this->node;
        }
        this->node = (DNameNode *) p;
        if (p == 0) {
            goto error;
        }
    } else {
error:
        this->status = 3;
    }
}
