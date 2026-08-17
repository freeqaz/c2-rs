// g02 — one unreferenced inline function, nothing else. Prediction: c2 emits
// nothing; obj is the bare 720 B four-section shell.
inline int gi(int x) { return x + 1; }
