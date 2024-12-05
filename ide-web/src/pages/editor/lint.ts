import { Diagnostic, linter } from "@codemirror/lint";
import { Text } from "@codemirror/text";
import { runDiagnostics } from "services/server";
import { currentFile } from "./Editor";

const ibLinter = linter(async (view) => {
    console.log(currentFile);
    if (currentFile == null) return [];

    const doc = view.state.doc;
    const ibDiagnostics = await runDiagnostics(doc.toString(), currentFile);
    console.log(ibDiagnostics);

    const diagnostics = ibDiagnostics.map((d) => {
        // const from = charOffset(doc, d.line, d.col);
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
