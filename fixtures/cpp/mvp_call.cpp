// W4a: single-function single-external tail call. `f` tail-calls the external
// `g`, compiling to one relative branch (`b g`) with an IMAGE_REL_PPC_REL24
// relocation to g's undefined external symbol. First fixture with a relocation
// and an undefined external symbol. See docs/OBJ_FORMAT_MVP.md (relocations).
extern void g();
void f() { g(); }
