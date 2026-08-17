// w-npos — the DANGER cell, kept as a refusal fence: an explicit class
// template instantiation emits a member function with zero ordinary roots
// (`?m@?$C@D@@QAAHXZ`, 8 bytes) through a module-level statement block. The
// provide-data recognizer must refuse this TU (its block content is neither a
// bare intro nor a `4F 1F` segment), because accepting it would emit a shell
// with no `.text` — a wrong obj. Expected verdict: NotImplemented, never Match.
template <class T> struct C { int m() { return 1; } };
template struct C<char>;
