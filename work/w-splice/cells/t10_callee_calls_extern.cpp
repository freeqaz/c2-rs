// GRID-T cell t10_callee_calls_extern — FIRES — and ?f's single REL24 must name ext, not g. Both bodies are the word 48000000, so the RELOCATION is the verdict (#882, w-seq s12)
void ext();
void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
