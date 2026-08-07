// GRID-W cell w01_chain1 — DEPTH 1 — c2 must name ext from ?f. Both bodies are the word 48000000, so the RELOCATION is the entire verdict (#882)
void ext();
void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
