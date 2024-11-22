import { Diagnostic, linter } from "@codemirror/lint";
import { Text } from "@codemirror/text";
import { runDiagnostics } from "services/server";
import { currentFile } from "./Editor";

const charOffset = (doc: Text, line: number, col: number) => {
    let offset = 0;
    for (let i = 1; i < line; i++) {
        console.log(doc.line(i).length);
        offset += doc.line(i).length + 1;
    }

    return offset + (col - 1);
};

const ibLinter = linter(async (view) => {
    if (currentFile == null) return [];

    const doc = view.state.doc;
    const ibDiagnostics = await runDiagnostics(doc.toString(), currentFile);

    const diagnostics = ibDiagnostics.map((d) => {
        console.log(`l: ${d.line}, c: ${d.col}`);
        const from = charOffset(doc, d.line, d.col);
        const diagnostic: Diagnostic = {
            from: from,
            to: from + 1,
            severity: "error",
            source: "ibc",
            message: d.message,
        };

        return diagnostic;
    });

    return diagnostics;
});

export default ibLinter;
