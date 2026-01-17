// ibSupport pulls in the linter, which reaches axios and firebase; neither can
// be transformed by jest, and indentation never touches the linter
jest.mock("./lint", () => ({ __esModule: true, default: [] }));

import { EditorState } from "@codemirror/state";
import { ensureSyntaxTree } from "@codemirror/language";
import { getIndent, ib } from "./ibSupport";

/// The indentation the editor gives the line holding `marker`.
function indentAt(doc: string, marker: string): number {
    const state = EditorState.create({ doc, extensions: [ib()] });
    const tree = ensureSyntaxTree(state, doc.length, 5000)!;

    return getIndent(tree, doc.indexOf(marker), 4);
}

describe("indentation", () => {
    /// A link of an else if chain sits level with the if it continues, so its
    /// body indents once, like the first branch.
    it("indents an else if branch like the first branch", () => {
        const doc =
            "if a then\n    output FIRST\nelse if b then\n    output SECOND\nend";

        expect(indentAt(doc, "FIRST")).toBe(4);
        expect(indentAt(doc, "SECOND")).toBe(4);
    });

    it("indents a nested if a level deeper", () => {
        const doc =
            "if a then\n    output FIRST\nelse\n    if b then\n        output NESTED\n    end\nend";

        expect(indentAt(doc, "NESTED")).toBe(8);
    });
});
