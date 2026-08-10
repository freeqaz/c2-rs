class App { int _pad; public: App(int, char **); ~App(); void Run(); };
void f2(int argc, char **argv) { App app(argc, argv); app.Run(); }
int main(int argc, char **argv) { App app(argc, argv); app.Run(); }
