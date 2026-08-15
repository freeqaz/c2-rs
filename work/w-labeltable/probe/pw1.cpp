int P(const unsigned char* s){ int r=0; for (const unsigned char* p=s; *p; p++) r=r+*p; return r; }
