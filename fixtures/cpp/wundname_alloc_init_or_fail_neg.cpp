// W-UNDNAME — the NEGATIVE controls for `alloc-init-or-fail`.
//
// Eight cells, each one clause away from `wundname_alloc_init_or_fail.cpp`, and
// **every one must be out of class**. A `_neg` file whose cells all decline for
// the SAME reason is a control that tests one clause eight times, which is why
// each cell's clause was read individually with a probe patch (applied, run and
// reverted — `work/w-undname/decline_probe.md`, board #1704's defect and
// w-cfgclass §6.2's method).
//
// Compile at `/O1 /Oi /EHsc /GR`, the class's own mode. At `/Ox` every cell is
// out of class for a ninth reason — the mode gate in the parser — which is the
// positive fixture's second row and not a cell here.

struct DNameNode;

struct HeapManager {
    void *getMemory(int size, int flags);
    void *getMemory3(int size, int flags, int extra);
};

void *plainGetMemory(int size, int flags);

extern HeapManager gHeapManager;
extern void *pairNode_vtable;

struct pairNode {
    void **vtable;
    const DNameNode *left;
    DNameNode *right;
    int refcount;
};

// n1 — the tested value is a SIGNED int, so c2 emits `cmpwi` where this class
// has three `cmplwi`s. The right program, three wrong words.
struct N1 {
    DNameNode *node;
    unsigned char status;
    void append(int n);
};

void N1::append(int n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right = 0;
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

// n2 — the allocation is a FREE function, so there is no object whose address
// becomes the call's `this`: one data symbol instead of two, r3 written from
// the first explicit argument, and a different argument schedule entirely
// (board #870's regime).
struct N2 {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n);
};

void N2::append(DNameNode *n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) plainGetMemory(16, 0);
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

// n3 — THREE explicit call arguments, so the setup is three `li`s and the
// `addi` takes a different slot in it.
struct N3 {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n);
};

void N3::append(DNameNode *n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory3(16, 0, 1);
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

// n4 — the status field is a WORD. The one instruction in this body whose
// OPCODE a type decides: `stw` writes three neighbouring fields to zero, links,
// and is a different program.
struct N4 {
    DNameNode *node;
    int status;
    void append(DNameNode *n);
};

void N4::append(DNameNode *n)
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

// n5 — the `lwz` inside the block and the link `stw` name DIFFERENT members, so
// the class would need two displacements where it has one field.
struct N5 {
    DNameNode *node;
    DNameNode *other;
    unsigned char status;
    void append(DNameNode *n);
};

void N5::append(DNameNode *n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right = n;
            p->refcount = -1;
            p->vtable = &pairNode_vtable;
            p->left = this->other;
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

// n6 — the third store writes a LITERAL instead of the second data symbol's
// address: one REFHI/REFLO quad instead of two, and the externals no longer
// interleave at all.
struct N6 {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n);
};

void N6::append(DNameNode *n)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right = n;
            p->refcount = -1;
            p->vtable = 0;
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

// n7 — TWO explicit formals. Both parks and the whole argument setup are
// functions of the arity, so this is a different body from the first word on.
struct N7 {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n, DNameNode *m);
};

void N7::append(DNameNode *n, DNameNode *m)
{
    if (n != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right = m;
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

// n8 — the entry test compares against a NON-ZERO literal, so its `cmplwi`
// carries an immediate this class has no field for.
struct N8 {
    DNameNode *node;
    unsigned char status;
    void append(DNameNode *n);
};

void N8::append(DNameNode *n)
{
    if (n != (DNameNode *) 4) {
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
