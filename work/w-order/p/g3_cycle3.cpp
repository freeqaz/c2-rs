void b(int);void c(int);
void a(int n){b(n);}
void b(int n){c(n);}
void c(int n){a(n);}
