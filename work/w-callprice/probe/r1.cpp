// w-callprice — R1 PROBE (scratch, never a fixture).
//
// Validates that the R1 counterfactual (`C2RS_WCP_R1=1`) is actually REACHED:
// a statement-call sequence whose later statement's receiver is a NAMED DATA
// OBJECT. Without the widening the second statement raises `call-token-0x26`;
// with it, the sequence completes.
//
// A probe, not a fixture: this lane ships no accepted class.

struct S {
    void a();
    void b();
    void set(int);
};

S gS;
S gT;

// The first statement's receiver is a named object too (mcall_tail's W-ADJUST
// arm already reads that); the SECOND is the one `eat_member_stmt_call` refuses.
void wcp_obj_two() {
    gS.a();
    gS.b();
}

void wcp_obj_three() {
    gS.a();
    gS.b();
    gS.set(3);
}

// A pointer receiver first, a named object second — the pure `eat_member_stmt_call`
// case with nothing else different.
void wcp_ptr_then_obj(S *s) {
    s->a();
    gS.b();
}

// Two distinct named objects.
void wcp_two_objects() {
    gS.a();
    gT.b();
}
