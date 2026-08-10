class App { int _pad; public: App(int, char **); ~App(); void Run(); };
extern int q1; extern int q2; extern int q3;
int main(int argc, char **argv) { App app(argc, argv); app.Run(); }
