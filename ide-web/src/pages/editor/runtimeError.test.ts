import { EditorState } from "@codemirror/state";
import { EditorView } from "@uiw/react-codemirror";
import runtimeErrorHighlight, { RuntimeErrorRange } from "./runtimeError";

/// The ranges the highlight decorates in a document of `doc`.
function markedRanges(
    doc: string,
    error: RuntimeErrorRange | null,
): Array<[number, number]> {
    const state = EditorState.create({
        doc,
        extensions: [runtimeErrorHighlight(error)],
    });

    const ranges: Array<[number, number]> = [];

    for (const value of state.facet(EditorView.decorations)) {
        if (typeof value == "function") continue;

        const cursor = value.iter();
        while (cursor.value != null) {
            ranges.push([cursor.from, cursor.to]);
            cursor.next();
        }
    }

    return ranges;
}

describe("runtime error highlight", () => {
    it("marks the range the error came from", () => {
        const doc = "output 1 / 0";
        expect(markedRanges(doc, { start: 7, end: 12 })).toEqual([[7, 12]]);
    });

    it("marks nothing without an error", () => {
        expect(markedRanges("output 1", null)).toEqual([]);
    });

    /// The error belongs to the source that was run, which the document may
    /// have moved past, so a stale range must not throw.
    it("clamps a range that runs past the end of the document", () => {
        expect(markedRanges("abcde", { start: 3, end: 99 })).toEqual([[3, 5]]);
        expect(markedRanges("abc", { start: 10, end: 20 })).toEqual([]);
    });

    it("still shows something for an empty range", () => {
        expect(markedRanges("abcde", { start: 2, end: 2 })).toEqual([[2, 3]]);
    });
});
