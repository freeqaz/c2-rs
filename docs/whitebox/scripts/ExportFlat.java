/* ExportFlat.java — Ghidra headless post-script.
 *
 * PROVENANCE: this script reads a disassembled third-party binary and writes
 * flat text exports for navigation. Its OUTPUT is disassembly-derived and must
 * not be pasted into crates/ without a disclosure entry (docs/whitebox/DISCLOSURE.md).
 *
 * Writes, into the directory named by the single script argument:
 *   functions.tsv   addr  size  name  nparams  ncallers  ncallees  nrefs  thunk  stackframe
 *   strings.tsv     addr  len   nxref  xrefs(csv, up to 24)  value(escaped)
 *   xrefs.tsv       from  to    type   from_func
 *   calls.tsv       caller_addr  caller_name  callee_addr  callee_name
 *   symbols.tsv     addr  name  source  namespace
 *   data.tsv        addr  size  type  nxref  xrefs(csv up to 12)  repr
 *   decomp_all.c    every function's decompiled C, separated by "// ===== FUNC <addr> <name> ====="
 *
 * @category Analysis
 */
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.data.*;
import ghidra.program.model.mem.*;
import ghidra.app.decompiler.*;
import java.io.*;
import java.util.*;

public class ExportFlat extends GhidraScript {

    private String esc(String s) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c == '\n') b.append("\\n");
            else if (c == '\r') b.append("\\r");
            else if (c == '\t') b.append("\\t");
            else if (c == '\\') b.append("\\\\");
            else if (c < 0x20 || c > 0x7e) b.append(String.format("\\x%02x", (int) c));
            else b.append(c);
        }
        return b.toString();
    }

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        File out = new File(args.length > 0 ? args[0] : "/tmp/export");
        out.mkdirs();
        Program p = currentProgram;
        Listing lst = p.getListing();
        ReferenceManager rm = p.getReferenceManager();
        FunctionManager fm = p.getFunctionManager();
        SymbolTable st = p.getSymbolTable();

        // ---- functions ----
        PrintWriter fw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "functions.tsv"))));
        fw.println("addr\tsize\tname\tnparams\tncallers\tncallees\tnrefs\tthunk\tframesize");
        List<Function> funcs = new ArrayList<>();
        for (Function f : fm.getFunctions(true)) funcs.add(f);
        for (Function f : funcs) {
            Address a = f.getEntryPoint();
            int nrefs = 0;
            for (Reference r : rm.getReferencesTo(a)) nrefs++;
            fw.printf("%s\t%d\t%s\t%d\t%d\t%d\t%d\t%s\t%d%n",
                a, f.getBody().getNumAddresses(), f.getName(), f.getParameterCount(),
                f.getCallingFunctions(monitor).size(), f.getCalledFunctions(monitor).size(),
                nrefs, f.isThunk() ? "thunk" : "-",
                f.getStackFrame() == null ? -1 : f.getStackFrame().getFrameSize());
        }
        fw.close();
        println("functions: " + funcs.size());

        // ---- calls ----
        PrintWriter cw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "calls.tsv"))));
        cw.println("caller_addr\tcaller_name\tcallee_addr\tcallee_name");
        for (Function f : funcs) {
            for (Function g : f.getCalledFunctions(monitor)) {
                cw.printf("%s\t%s\t%s\t%s%n", f.getEntryPoint(), f.getName(), g.getEntryPoint(), g.getName());
            }
        }
        cw.close();

        // ---- strings + data ----
        PrintWriter sw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "strings.tsv"))));
        sw.println("addr\tlen\tnxref\txrefs\txref_funcs\tvalue");
        PrintWriter dw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "data.tsv"))));
        dw.println("addr\tsize\ttype\tnxref\txrefs\trepr");
        DataIterator di = lst.getDefinedData(true);
        int nstr = 0, ndata = 0;
        while (di.hasNext()) {
            Data d = di.next();
            DataType dt = d.getDataType();
            Address a = d.getAddress();
            List<String> xr = new ArrayList<>();
            List<String> xf = new ArrayList<>();
            int nx = 0;
            for (Reference r : rm.getReferencesTo(a)) {
                nx++;
                if (xr.size() < 24) {
                    xr.add(r.getFromAddress().toString());
                    Function ff = fm.getFunctionContaining(r.getFromAddress());
                    if (ff != null && !xf.contains(ff.getEntryPoint().toString()))
                        xf.add(ff.getEntryPoint().toString());
                }
            }
            boolean isStr = (dt instanceof StringDataType) || (dt instanceof TerminatedStringDataType)
                || (dt instanceof UnicodeDataType) || (dt instanceof TerminatedUnicodeDataType)
                || dt.getName().toLowerCase().contains("string") || dt.getName().toLowerCase().contains("unicode");
            if (isStr) {
                Object v = d.getValue();
                String s = v == null ? "" : v.toString();
                sw.printf("%s\t%d\t%d\t%s\t%s\t%s%n", a, d.getLength(), nx,
                    String.join(",", xr), String.join(",", xf), esc(s));
                nstr++;
            } else {
                String repr;
                try { Object v = d.getValue(); repr = v == null ? "" : esc(String.valueOf(v)); }
                catch (Exception e) { repr = ""; }
                dw.printf("%s\t%d\t%s\t%d\t%s\t%s%n", a, d.getLength(), dt.getName(), nx,
                    String.join(",", xr.subList(0, Math.min(12, xr.size()))), repr);
                ndata++;
            }
        }
        sw.close(); dw.close();
        println("strings: " + nstr + " data: " + ndata);

        // ---- symbols ----
        PrintWriter yw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "symbols.tsv"))));
        yw.println("addr\tname\tsource\ttype\tnamespace");
        for (Symbol s : st.getAllSymbols(false)) {
            yw.printf("%s\t%s\t%s\t%s\t%s%n", s.getAddress(), s.getName(),
                s.getSource(), s.getSymbolType(), s.getParentNamespace().getName(true));
        }
        yw.close();

        // ---- xrefs (code refs only, to keep it bounded) ----
        PrintWriter xw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "xrefs.tsv"))));
        xw.println("from\tto\ttype\tfrom_func");
        AddressIterator ai = rm.getReferenceSourceIterator(p.getMemory(), true);
        long nx = 0;
        while (ai.hasNext()) {
            Address a = ai.next();
            Function ff = fm.getFunctionContaining(a);
            for (Reference r : rm.getReferencesFrom(a)) {
                xw.printf("%s\t%s\t%s\t%s%n", a, r.getToAddress(), r.getReferenceType(),
                    ff == null ? "-" : ff.getEntryPoint().toString());
                nx++;
            }
        }
        xw.close();
        println("xrefs: " + nx);

        // ---- decompile everything ----
        DecompInterface di2 = new DecompInterface();
        DecompileOptions opts = new DecompileOptions();
        di2.setOptions(opts);
        di2.toggleCCode(true);
        di2.toggleSyntaxTree(true);
        di2.setSimplificationStyle("decompile");
        di2.openProgram(p);
        PrintWriter pw = new PrintWriter(new BufferedWriter(new FileWriter(new File(out, "decomp_all.c")), 1 << 20));
        int ok = 0, fail = 0;
        for (Function f : funcs) {
            if (monitor.isCancelled()) break;
            pw.printf("// ===== FUNC %s %s size=%d =====%n", f.getEntryPoint(), f.getName(),
                f.getBody().getNumAddresses());
            try {
                DecompileResults res = di2.decompileFunction(f, 45, monitor);
                if (res != null && res.decompileCompleted() && res.getDecompiledFunction() != null) {
                    pw.print(res.getDecompiledFunction().getC());
                    ok++;
                } else {
                    pw.printf("// DECOMPILE FAILED: %s%n", res == null ? "null" : res.getErrorMessage());
                    fail++;
                }
            } catch (Exception e) {
                pw.printf("// DECOMPILE EXCEPTION: %s%n", e);
                fail++;
            }
            if ((ok + fail) % 250 == 0) { pw.flush(); println("decomp " + (ok + fail) + "/" + funcs.size()); }
        }
        pw.close();
        di2.dispose();
        println("decomp ok=" + ok + " fail=" + fail);
    }
}
