// diag.cpp — lane wb-select (WB-I).  POST-GRID DIAGNOSTICS, UNSCORED.
//
// Run AFTER select_grid.cpp was frozen, compiled and scored.  Nothing here is
// a prediction; these cells exist only to localise the two mechanism MISSES
// (S11's mask form and S12's missing branch) so the findings doc can say what
// is unread rather than guessing.  Recorded so the reader can tell scored
// evidence from diagnostic evidence.
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc

extern "C" {
int  dsel_ext(int);

// --- D-A: the rlandi mask form.  Which masks become rlwinm and which become
//          li + and?
int d_mask2 (unsigned x){ return x < 10u ? 2 : 0; }
int d_mask4 (unsigned x){ return x < 10u ? 4 : 0; }
int d_mask8 (unsigned x){ return x < 10u ? 8 : 0; }
int d_mask16(unsigned x){ return x < 10u ? 16 : 0; }
int d_mask3 (unsigned x){ return x < 10u ? 3 : 0; }
int d_mask4b(unsigned x){ return x < 10u ? 7 : 3; }   // = grid S1, mask 4, bias 3
int d_mask8b(unsigned x){ return x < 10u ? 11 : 3; }  // mask 8, bias 3

// --- D-B: is the branch really gone, or was S12 special?
int d_if_u_call (unsigned x){ if (x < 10u) return dsel_ext(1); return 2; }
int d_if_u_3arm (unsigned x){ if (x < 10u) return 1; if (x < 20u) return 2; return 3; }
int d_if_u_store(unsigned x, int *p){ if (x < 10u) { *p = 1; return 1; } return 2; }
int d_if_s      (int x){ if (x < 10) return 1; return 2; }
int d_if_u_big  (unsigned x){ if (x < 100000u) return 1; return 2; }
int d_while_u   (unsigned x){ int s = 0; while (x < 10u) { s += (int)x; ++x; } return s; }
}
