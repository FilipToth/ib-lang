import { strict as assert } from "assert";
import { parser } from "../src/parser.js";

/// Collects the names of every error node lezer produced, so a test can assert
/// a snippet parsed cleanly.
function errorNodes(code) {
    const tree = parser.parse(code);
    const errors = [];
    const cursor = tree.cursor();

    do {
        if (cursor.type.isError) {
            errors.push(`${cursor.from}..${cursor.to}`);
        }
    } while (cursor.next());

    return errors;
}

/// Collects every node name in the tree, used to check an operator was
/// recognised as the node type we expect rather than swallowed as an
/// identifier.
function nodeNames(code) {
    const tree = parser.parse(code);
    const names = [];
    const cursor = tree.cursor();

    do {
        names.push(cursor.type.name);
    } while (cursor.next());

    return names;
}

function assertParses(code) {
    const errors = errorNodes(code);
    assert.deepEqual(errors, [], `${JSON.stringify(code)} should parse cleanly`);
}

describe("operators", () => {
    it("parses the symbol operators", () => {
        for (const op of ["+", "-", "*", "/", "==", "!=", ">=", "<="]) {
            assertParses(`A ${op} B`);
        }
    });

    it("tags the new symbol operators as MiscOperator", () => {
        for (const op of ["!=", ">=", "<="]) {
            const names = nodeNames(`A ${op} B`);
            assert.ok(
                names.includes("MiscOperator"),
                `${op} should be a MiscOperator, got ${names.join(",")}`
            );
        }
    });

    it("parses the word operators in either case", () => {
        for (const op of ["and", "AND", "or", "OR", "mod", "MOD", "div", "DIV"]) {
            assertParses(`A ${op} B`);
        }
    });

    it("tags the word operators as their own keyword nodes", () => {
        const cases = [
            ["and", "AndKeyword"],
            ["AND", "AndKeyword"],
            ["or", "OrKeyword"],
            ["OR", "OrKeyword"],
            ["mod", "ModKeyword"],
            ["MOD", "ModKeyword"],
            ["div", "DivKeyword"],
            ["DIV", "DivKeyword"],
        ];

        for (const [op, expected] of cases) {
            const names = nodeNames(`A ${op} B`);
            assert.ok(
                names.includes(expected),
                `${op} should produce ${expected}, got ${names.join(",")}`
            );
        }
    });

    it("still treats NOT as a keyword", () => {
        const names = nodeNames("NOT A");
        assert.ok(names.includes("NotKeyword"));
    });

    it("does not mistake identifiers that start with a word operator", () => {
        for (const name of ["android", "order", "modulo", "divide"]) {
            const names = nodeNames(`${name} = 1`);
            assert.ok(
                names.includes("Identifier"),
                `${name} should stay an Identifier, got ${names.join(",")}`
            );
        }
    });

    it("parses generic instantiation alongside the new <= and >= tokens", () => {
        assertParses("S = new Stack<Int>()");
        assertParses("if A <= B then\n  output A\nend");
        assertParses("if A >= B then\n  output A\nend");
    });

    it("parses a condition combining several new operators", () => {
        assertParses("if A >= 1 AND B <= 2 OR NOT C == 3 then\n  output A\nend");
        assertParses("output 15 mod 7");
        assertParses("output 15 div 7");
    });
});
