import { Diagnostic, linter } from "@codemirror/lint";
import { runDiagnostics } from "services/server";

const ibLinter = linter(async (view) => {
    const ibDiagnostics = await runDiagnostics(view.state.doc.toString());

    const diagnostics = ibDiagnostics.map((d) => {
        const diagnostic: Diagnostic = {
            from: d.offset_start,
            to: d.offset_end,
            severity: "error",
            source: "ibc",
            message: d.message,
        };

        return diagnostic;
    });

    return diagnostics;
});

export default ibLinter;
