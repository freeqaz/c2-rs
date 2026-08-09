struct P { char *mFree; };
void wpool_cast_store_only(P *p, void *v) { *(void **)v = p->mFree; }
