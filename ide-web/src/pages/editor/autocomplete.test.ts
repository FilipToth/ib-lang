// ibSupport pulls in the linter, which reaches axios, firebase and thememirror
// -- all ESM that CRA's jest cannot transform. Completion never uses the
// linter, so stubbing it out keeps the language definition intact.
jest.mock("./lint", () => ({ __esModule: true, default: [] }));

import { EditorState } from "@codemirror/state";
import { CompletionContext } from "@codemirror/autocomplete";
import { ensureSyntaxTree } from "@codemirror/language";
import { ib } from "./ibSupport";
import ibCompletions from "./autocomplete";

/// Builds an editor state over `doc`, forces a full parse, and asks for the
/// completions offered at `pos` (default: end of document).
function completionsAt(doc: string, pos?: number): string[] {
    const state = EditorState.create({ doc, extensions: [ib()] });
    const at = pos ?? doc.length;

    // the language parses incrementally; force it so the tree is complete
    ensureSyntaxTree(state, state.doc.length, 5000);

    const context = new CompletionContext(state, at, true);
    const result = ibCompletions(context);
    if (result == null) return [];

    return result.options.map((o) => o.label);
}

describe("autocomplete symbol resolution", () => {
    it("suggests a variable declared before the cursor", () => {
        const labels = completionsAt("ALPHA = 1\n");
        expect(labels).toContain("ALPHA");
    });

    it("suggests variables declared after a non-declaration statement", () => {
        // `output ALPHA` is neither a VariableAssignment nor a
        // FunctionDeclaration, so it must not stop the scope walk
        const labels = completionsAt("ALPHA = 1\noutput ALPHA\nBETA = 2\n");
        expect(labels).toContain("ALPHA");
        expect(labels).toContain("BETA");
    });

    it("suggests variables declared after an if statement", () => {
        const doc = "ALPHA = 1\nif true then\n  output ALPHA\nend\nBETA = 2\n";
        const labels = completionsAt(doc);
        expect(labels).toContain("BETA");
    });

    it("suggests a function declared after another statement", () => {
        const doc = "ALPHA = 1\noutput ALPHA\nfunction doThing()\n  output 1\nend\n";
        const labels = completionsAt(doc);
        expect(labels).toContain("doThing");
    });

    it("does not crash when the cursor sits at the document root", () => {
        // a blank final line resolves to the Program node, which has no parent
        expect(() => completionsAt("ALPHA = 1\n")).not.toThrow();
        expect(() => completionsAt("")).not.toThrow();
    });

    it("does not leak collection methods into top-level completions", () => {
        // push/pop belong to a Stack instance, not to the enclosing scope
        const labels = completionsAt("S = new Stack<Int>()\n");
        expect(labels).toContain("S");
        expect(labels).not.toContain("push");
    });
});

describe("autocomplete member access", () => {
    // one case per collection type: the declaration, a partial member name to
    // complete from, and the methods the binder exposes on that type
    const cases: Array<[string, string, string[]]> = [
        ["Stack<Int>", "p", ["push", "pop", "isEmpty"]],
        ["Queue<Int>", "e", ["enqueue", "dequeue", "isEmpty"]],
        [
            "Collection<Int>",
            "a",
            ["addItem", "getNext", "hasNext", "resetNext", "isEmpty"],
        ],
        ["Array<Int>", "g", ["push", "get", "len", "isEmpty"]],
    ];

    it.each(cases)("offers %s methods after a dot", (type, prefix, methods) => {
        const labels = completionsAt(`V = new ${type}()\nV.${prefix}`);

        for (const method of methods) {
            expect(labels).toContain(method);
        }
    });
});
