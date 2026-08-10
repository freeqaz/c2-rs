class App { int _pad; public: App(int, char **); ~App(); void Run(); };
int main(int argc, char **argv) { App app(argc, argv); app.Run(); return 0; }
