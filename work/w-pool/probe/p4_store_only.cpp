struct P { char *mFree; };
void wpool_store_only(P *p, void *v) { p->mFree = (char *)v; }
