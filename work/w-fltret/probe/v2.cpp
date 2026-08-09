// w-fltret probe v2 — the FREE-FUNCTION float value tail of a statement-call
// sequence. `parse_call_sequence_from`'s value arm reads `eat_call_head` and
// never consults its `CallRet`, so this shape is admitted as
// `SeqTail::CallValue { add_k: 0 }` with nothing marking the body FP-touching.
// If that is what happens, the obj is one symbol short of c2's — `_fltused`.
void  g1();
float gf();
int   gi();
double gd();

// the int control — accepted today, and correctly carries no `_fltused`
int s_int() { g1(); return gi(); }

// THE SUSPECT: a float-returning free call as the sequence's value tail
float s_float() { g1(); return gf(); }

// the same at double width
double s_double() { g1(); return gd(); }
