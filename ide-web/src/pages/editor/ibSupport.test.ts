// ibSupport pulls in the linter, which reaches axios and firebase; neither can
// be transformed by jest, and indentation never touches the linter
jest.mock("./lint", () => ({ __esModule: true, default: [] }));

import { EditorState } from "@codemirror/state";
import { ensureSyntaxTree } from "@codemirror/language";
import { getIndent, ib } from "./ibSupport";

/// The indentation the editor gives the line holding `marker`.
function indentAt(doc: string, marker: string): number {
    return indentAtPos(doc, doc.indexOf(marker));
}

/// The indentation the editor gives a new line at `pos`.
function indentAtPos(doc: string, pos: number): number {
    const state = EditorState.create({ doc, extensions: [ib()] });
    const tree = ensureSyntaxTree(state, doc.length, 5000)!;

    return getIndent(tree, pos, 4);
}

/// The indentation of the line opened by pressing enter at the end of `doc`,
/// which is where a program is usually being written.
function indentAfter(doc: string): number {
    return indentAtPos(doc + "\n", doc.length + 1);
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

    it("indents the body of each kind of loop", () => {
        const loops = [
            "loop for i from 1 to 3\n    output BODY\nend",
            "loop i from 1 to 3\n    output BODY\nend",
            "loop while a\n    output BODY\nend",
            "loop until a\n    output BODY\nend",
        ];

        for (const doc of loops) {
            expect(indentAt(doc, "BODY")).toBe(4);
        }
    });

    it("indents a loop nested in a function and an if", () => {
        const doc =
            "function f()\n    if a then\n        loop while b\n            output NESTED\n        end\n    end\nend";

        expect(indentAt(doc, "NESTED")).toBe(12);
    });

    /// Nothing follows the header yet to tell the parser where the body is,
    /// which is how a program is usually written: top to bottom.
    it("indents after a header with nothing after it yet", () => {
        expect(indentAfter("if a then")).toBe(4);
        expect(indentAfter("loop while a")).toBe(4);
        expect(indentAfter("loop until a")).toBe(4);
        expect(indentAfter("loop for i from 1 to 3")).toBe(4);
        expect(indentAfter("function f()")).toBe(4);
        expect(indentAfter("if a then\n    output A\nelse")).toBe(4);
    });

    it("keeps the body's indentation for the next line in it", () => {
        expect(indentAfter("if a then\n    output A")).toBe(4);
        expect(indentAfter("function f()\n    loop while b")).toBe(8);
    });

    /// The body is behind the cursor once the statement is closed.
    it("does not indent after a finished statement", () => {
        expect(indentAfter("if a then\n    output A\nend")).toBe(0);
        expect(indentAfter("loop while a\n    output A\nend")).toBe(0);
        expect(indentAfter("output 1")).toBe(0);
        expect(indentAfter("")).toBe(0);
        const nested = "function f()\n    if a then\n        output A\n    end";
        expect(indentAfter(nested)).toBe(4);
    });

    /// Pressing enter in a loop body, above its end, lines the new line up
    /// with the body rather than the loop.
    it("indents a new line in a loop body", () => {
        const doc = "loop while a\n    output A\n\nend";

        expect(indentAt(doc, "\nend")).toBe(4);
    });
});
