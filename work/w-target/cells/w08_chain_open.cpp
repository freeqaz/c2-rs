// GRID-W cell w08_chain_open — w-splice's S6-chain-open — the chain's end carries MORE THAN ONE call, so its target is not a single name. `close_target` must refuse with `callee-multi-call` rather than pick one
void ext1();
void ext2();
void h() { ext1(); ext2(); }
void g() { h(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
