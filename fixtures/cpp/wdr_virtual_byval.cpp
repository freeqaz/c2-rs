// wdr — virtual dispatch (`67`, `9A`) and the by-value return (`64`), the two
// opcodes that stood between the statement-layer scanner and 190,868 bodies.
//
// DECODE ONLY. Nothing here is lowered and nothing here is in class: every
// function must refuse at a named census key AND decode end to end on the
// control-flow axis. `c2rs census` prints both columns, which is what makes the
// pair checkable — a case that decodes but does not refuse would mean the
// scanner had been wired into acceptance, and a case that refuses but does not
// decode would mean a width is still wrong.
//
// The bytes each construct produces are transcribed into
// `crates/c2-il/src/func/body/shapes/control_flow.rs`'s tests; this file is what
// keeps them re-derivable, and what puts them through the capture-stability lane.

struct Val {
    int a, b, c;
};

// ---- the by-value return: `64 <TYPE>` ------------------------------------
// The construct is a call whose result is a class returned BY VALUE. The call
// stores into a `9B`-bound temporary, `2C` takes its address, and `64 <TYPE>`
// materializes into it. Found by elimination over a 27-function battery: of
// casts, pointer-to-member, new/delete, indirect calls, aggregate assignment
// and array subscript, this is the only one that emits the opcode.
struct Src {
    Val Make();
    virtual Val VMake();
    virtual int VGet();
    virtual int VSet(int);
    int m;
};

int use(const Val& v);
int useint(int v);

void byval_discard(Src* s) { s->Make(); }
int byval_member(Src* s) { return s->Make().a; }
int byval_arg(Src* s) { return use(s->Make()); }
int byval_two(Src* s, Src* t) { return s->Make().a + t->Make().b; }
Val byval_ret(Src* s) { return s->Make(); }
int byval_nested(Src* s) { return useint(s->Make().a + 1); }

// ---- virtual dispatch: `67 <varint slot> <tok>` then `9A <TYPE>` ---------
// `67`'s first field is the vtable BYTE offset, and it is a signed varint.
// Every witness this project had was below 0x80, where a varint and a plain
// byte are indistinguishable; `wide_32` below is the separator.
int virt_ptr(Src* s) { return s->VGet(); }
int virt_ref(Src& r) { return r.VGet(); }
int virt_arg(Src* s) { return s->VSet(3); }
void virt_discard(Src* s) { s->VGet(); }
// `67` and `64` in one body: a virtual call returning by value. Three
// independently established widths that must agree on one cursor.
void virt_byval(Src* s) { s->VMake(); }

struct Wide {
    virtual int v00(); virtual int v01(); virtual int v02(); virtual int v03();
    virtual int v04(); virtual int v05(); virtual int v06(); virtual int v07();
    virtual int v08(); virtual int v09(); virtual int v10(); virtual int v11();
    virtual int v12(); virtual int v13(); virtual int v14(); virtual int v15();
    virtual int v16(); virtual int v17(); virtual int v18(); virtual int v19();
    virtual int v20(); virtual int v21(); virtual int v22(); virtual int v23();
    virtual int v24(); virtual int v25(); virtual int v26(); virtual int v27();
    virtual int v28(); virtual int v29(); virtual int v30(); virtual int v31();
    virtual int v32(); virtual int v33(); virtual int v34(); virtual int v35();
    virtual int v36(); virtual int v37(); virtual int v38(); virtual int v39();
};

// Slot 31 is byte offset 0x7C — the last one the varint short form reaches.
int wide_31(Wide* p) { return p->v31(); }
// Slot 32 is byte offset 0x80, and emits `67 80 80 00 00 00 <tok>`. A
// plain-byte reading of the field desynchronizes here, on every class in the
// corpus with more than 32 virtual functions.
int wide_32(Wide* p) { return p->v32(); }
int wide_39(Wide* p) { return p->v39(); }
// Two dispatches in one statement: the second `67` is met with the cursor
// already advanced past the first, so a wrong width is caught twice over.
int wide_two(Wide* p, Wide* q) { return p->v32() + q->v33(); }
