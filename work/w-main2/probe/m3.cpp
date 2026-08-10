class App { int _pad; public: App(int, char **); ~App(); void Run(); };
int lf(int a) { return a + 1; }
int lg(int a) { return a + 2; }
int main(int argc, char **argv) { App app(argc, argv); app.Run(); }
