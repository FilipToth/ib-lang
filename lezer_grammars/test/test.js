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

/// Finds the text every Expression node covers, so a test can assert a
/// multi-token expression was kept whole instead of spilling into the Block
/// that follows it.
function expressionTexts(code) {
    const tree = parser.parse(code);
    const cursor = tree.cursor();
    const texts = [];

    do {
        if (cursor.type.name === "Expression") {
            texts.push(code.slice(cursor.from, cursor.to));
        }
    } while (cursor.next());

    return texts;
}

/// Asserts `code` parses cleanly and that `expr` came out as one Expression.
function assertWholeExpression(code, expr) {
    assertParses(code);

    const texts = expressionTexts(code);
    assert.ok(
        texts.includes(expr),
        `${JSON.stringify(expr)} should be one Expression, got ${JSON.stringify(texts)}`
    );
}

describe("loops", () => {
    it("parses a counted loop with and without the optional for", () => {
        const withFor = "loop for N from 0 to 5\n  output N\nend";
        const withoutFor = "loop N from 0 to 5\n  output N\nend";

        assertParses(withFor);
        assertParses(withoutFor);

        assert.ok(nodeNames(withFor).includes("ForKeyword"));

        const plain = nodeNames(withoutFor);
        assert.ok(!plain.includes("ForKeyword"));
        assert.ok(plain.includes("ForStatement"));
    });

    it("parses an until loop in either case", () => {
        for (const kw of ["until", "UNTIL"]) {
            assertParses(`loop ${kw} F > 3\n  output F\nend`);

            const names = nodeNames(`loop ${kw} F > 3\n  output F\nend`);
            assert.ok(
                names.includes("UntilKeyword"),
                `${kw} should produce UntilKeyword, got ${names.join(",")}`
            );
            assert.ok(names.includes("UntilStatement"));
        }
    });

    it("still parses while loops", () => {
        const names = nodeNames("loop while C < 3\n  output C\nend");
        assert.ok(names.includes("WhileKeyword"));
        assert.ok(names.includes("WhileStatement"));
    });

    it("does not mistake identifiers starting with a loop keyword", () => {
        for (const name of ["untilled", "forever", "whiles"]) {
            const names = nodeNames(`${name} = 1`);
            assert.ok(
                names.includes("Identifier"),
                `${name} should stay an Identifier, got ${names.join(",")}`
            );
        }
    });
});

describe("operators", () => {
    it("parses the symbol operators and tags them as MiscOperator", () => {
        for (const op of ["+", "-", "*", "/", "==", "!=", ">=", "<="]) {
            const source = `A ${op} B`;
            assertParses(source);

            const names = nodeNames(source);
            assert.ok(
                names.includes("MiscOperator"),
                `${op} should be a MiscOperator, got ${names.join(",")}`
            );
        }
    });

    it("parses the word operators in either case, each as its own node", () => {
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
            const source = `A ${op} B`;
            assertParses(source);

            const names = nodeNames(source);
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

    it("parses comparison operators", () => {
        for (const op of [">", "<", ">=", "<="]) {
            assertParses(`if A ${op} B then\n  output A\nend`);
        }
    });

    it("parses generic instantiation alongside the comparison tokens", () => {
        assertParses("S = new Stack<Int>()");
        assertParses("Q = new Queue<String>()");
        assertParses("if A <= B then\n  output A\nend");
        assertParses("if A >= B then\n  output A\nend");
    });

    it("parses a condition combining several new operators", () => {
        assertParses("if A >= 1 AND B <= 2 OR NOT C == 3 then\n  output A\nend");
        assertParses("output 15 mod 7");
        assertParses("output 15 div 7");
    });
});

describe("multi-token expressions", () => {
    /// A loop bound and a loop condition are both followed directly by the
    /// Block, with no delimiter keyword to close the expression. Without
    /// binary-expression structure the trailing tokens end up as statements
    /// inside the Block instead.
    it("keeps a loop bound whole when it spans several tokens", () => {
        assertWholeExpression("loop N from 0 to COUNT - 1\n  output N\nend", "COUNT - 1");
        assertWholeExpression("loop N from A + 1 to B * 2\n  output N\nend", "B * 2");
    });

    it("keeps a loop condition whole", () => {
        assertWholeExpression("loop while I < 10\n  output I\nend", "I < 10");
        assertWholeExpression("loop until F * F > NUM\n  output F\nend", "F * F > NUM");
    });

    it("keeps an if condition whole", () => {
        assertWholeExpression("if X >= 4 then\n  output X\nend", "X >= 4");
        assertWholeExpression("if A AND B OR C then\n  output A\nend", "A AND B OR C");
    });

    /// A '-' that opens an expression is still unary; only one that follows an
    /// operand binds as a binary operator.
    it("still reads a leading minus as unary", () => {
        assertWholeExpression("X = -1", "-1");
        assertWholeExpression("output -X", "-X");
        assertWholeExpression("output NOT true", "NOT true");
    });
});
