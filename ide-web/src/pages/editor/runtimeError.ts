import { Extension } from "@codemirror/state";
import { Decoration, EditorView } from "@uiw/react-codemirror";

/// Where a runtime error happened, as character offsets into the source the
/// server was given.
export interface RuntimeErrorRange {
    start: number;
    end: number;
}

const theme = EditorView.baseTheme({
    ".cm-runtimeError": {
        backgroundColor: "rgba(255, 82, 82, 0.25)",
        borderBottom: "2px solid #ff5252",
    },
});

const mark = Decoration.mark({ class: "cm-runtimeError" });

/// Highlights the expression a runtime error came from.
///
/// The decoration is built once from `error` rather than tracked through
/// document changes, because an edit invalidates the error itself -- the editor
/// clears it on the next change.
const runtimeErrorHighlight = (
    error: RuntimeErrorRange | null,
    docLength: number
): Extension => {
    if (error == null) return [];

    // the error came from a run of source the document may have moved past, so
    // the range is clamped rather than trusted
    const start = Math.min(error.start, docLength);
    const end = Math.min(Math.max(error.end, start + 1), docLength);

    // nothing left to point at
    if (start >= end) return [];

    const decorations = Decoration.set([mark.range(start, end)]);
    return [theme, EditorView.decorations.of(decorations)];
};

export default runtimeErrorHighlight;
