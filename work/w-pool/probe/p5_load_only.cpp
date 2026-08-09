struct P { char *mFree; };
void wpool_load_only(P *p, void **v) { *v = p->mFree; }
