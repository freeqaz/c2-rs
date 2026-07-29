// Scalar conversions (`.ex` opcode 0x2C), one per function, source operand is
// always a formal so nothing folds. Evidence base for docs/IL_CAST_CONVERT.md
// §2: which conversions c2 lowers to nothing and which to a real instruction.
int i2i(int a) { return a; }

unsigned int i2u(int a) { return (unsigned int)a; }

int u2i(unsigned int a) { return (int)a; }

char i2c(int a) { return (char)a; }

short i2s(int a) { return (short)a; }

unsigned char i2uc(int a) { return (unsigned char)a; }

int c2i(char a) { return a; }

int s2i(short a) { return a; }

int uc2i(unsigned char a) { return a; }

int b2i(bool a) { return a; }

float i2f(int a) { return (float)a; }

double i2d(int a) { return (double)a; }

int f2i(float a) { return (int)a; }

double f2d(float a) { return a; }

float d2f(double a) { return (float)a; }

long long i2ll(int a) { return a; }

int ll2i(long long a) { return (int)a; }

void *p2v(int *p) { return p; }

int *v2p(void *p) { return (int *)p; }
