import { Diagnostic, linter } from "@codemirror/lint";
import { Text } from "@codemirror/text";
import { runDiagnostics } from "services/server";
import { currentFile } from "./Editor";
import { syntaxTree } from "@codemirror/language";
import logTree from "./logTree";

const ibLinter = linter(async (view) => {
    if (currentFile == null) return [];

    // try with and without doc see if updates correctly
    const ibDiagnostics = await runDiagnostics(currentFile);

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
