import { Completion, CompletionContext } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { SyntaxNode, Tree } from "@lezer/common";
import { EditorView, TransactionSpec } from "@uiw/react-codemirror";
import { getIndent } from "./ibSupport";
import { Text } from "@codemirror/text";

interface Symbol {
    name: string;
    kind: string;
    type: string | null;
}

/// Methods each collection type exposes. Mirrors the analyzer's own tables in
/// core/ibc/src/analysis/binding/types.rs -- the two must stay in step, or the
/// editor will suggest methods the binder rejects.
const TYPE_METHODS: { [type: string]: string[] } = {
    Array: ["push", "get", "len", "isEmpty"],
    Collection: ["addItem", "getNext", "hasNext", "resetNext", "isEmpty"],
    Stack: ["push", "pop", "isEmpty"],
    Queue: ["enqueue", "dequeue", "isEmpty"],
};

const getTypeSymbols = (type: string | null): Symbol[] => {
    if (type == null) return [];

    const methods = TYPE_METHODS[type];
    if (methods == null) return [];

    const symbols: Symbol[] = [];
    for (const method of methods) {
        symbols.push({ name: method, kind: "function", type: null });
    }

    return symbols;
};

const getMemberExprSymbols = (
    identifier: SyntaxNode,
    doc: Text,
    existingSymbols: Symbol[],
) => {
    // expr.identifier
    const expr = identifier.prevSibling!;

    // actual expressio, e.g. reference expression
    const actualExpr = expr.firstChild;
    if (actualExpr == null) return [];

    if (actualExpr.name != "ReferenceExpression") {
        // resolve symbol
        const prevText = doc.sliceString(actualExpr.from, actualExpr.to);
        const matching = existingSymbols.filter((s) => s.name == prevText);

        if (matching.length == 0) return [];

        const first = matching[0];
        const typeMethods = getTypeSymbols(first.type);
        return typeMethods;
    }

    return [];
};

const resolveSymbols = (
    tree: Tree,
    context: CompletionContext,
    word: string | undefined,
) => {
    const nodeBefore = tree.resolveInner(context.pos, -1);

    const scopes: SyntaxNode[] = [];
    getScopesRecursive(nodeBefore, scopes);

    const symbols: Symbol[] = [];
    for (const scope of scopes) {
        if (scope.firstChild == null) continue;

        checkForParameters(scope, context, symbols);
        resolveSymbolsInScope(scope.firstChild, context, symbols);
    }

    // at the top level of the document the resolved node is the root, which has
    // no parent, so this cannot use a non-null assertion
    const parentNode = nodeBefore.parent;
    if (parentNode != null && parentNode.name == "MemberAccessExpression") {
        // nodeBefore is identifier
        const typeSymbols = getMemberExprSymbols(
            nodeBefore,
            context.state.doc,
            symbols,
        );

        symbols.push(...typeSymbols);
    }

    const resolvedSymbols: Symbol[] = [];
    symbols.forEach((symbol) => {
        if (symbol.name == word) return;

        resolvedSymbols.push(symbol);
    });

    return resolvedSymbols;
};

const getScopesRecursive = (node: SyntaxNode, scopes: SyntaxNode[]) => {
    if (node.name == "Block" || node.name == "Program") scopes.push(node);

    if (node.parent == null) return;

    getScopesRecursive(node.parent, scopes);
};

const resolveSymbolsInScope = (
    scopeChild: SyntaxNode,
    context: CompletionContext,
    symbols: Symbol[],
) => {
    // a scope will always contain atoms, the first
    // child of the atom is the actual node
    const node = scopeChild.firstChild;

    if (node != null) {
        const identifierNode = node.getChild("Identifier");
        const identifier = context.state.sliceDoc(
            identifierNode?.from,
            identifierNode?.to,
        );

        const document = context.state.doc;

        let kind = null;
        let type = null;

        if (node.name == "VariableAssignment") {
            kind = "variable";

            // record the declared type on the symbol so member access can
            // resolve its methods later. this must not shadow the outer
            // binding, and the methods themselves are not scope symbols.
            type = getVariableDeclarationType(document, node);
        } else if (node.name == "FunctionDeclaration") {
            kind = "function";
        }

        if (identifier != null && kind != null) {
            symbols.push({ name: identifier, kind: kind, type: type });
        }
    }

    // a statement that declares nothing must not stop the walk: declarations
    // after it are still in scope
    if (scopeChild.nextSibling == null) return;

    resolveSymbolsInScope(scopeChild.nextSibling, context, symbols);
};

/// The outermost name of a `Type` node, which is what TYPE_METHODS is keyed
/// by: `Array` for `Array<Stack<Int>>`.
const getTypeName = (document: Text, typeNode: SyntaxNode | null) => {
    const nameNode = typeNode?.getChild("TypeAnnotation");
    if (nameNode == null) return null;

    return document.sliceString(nameNode.from, nameNode.to);
};

const getVariableDeclarationType = (document: Text, varNode: SyntaxNode) => {
    const expr = varNode.getChild("Expression");
    if (expr == null) return null;

    const objInstantiation = expr.getChild("ObjectInstantiationExpression");
    if (objInstantiation == null) return null;

    return getTypeName(document, objInstantiation.getChild("Type"));
};

const checkForParameters = (
    block: SyntaxNode,
    context: CompletionContext,
    symbols: Symbol[],
) => {
    // parameters are in scope in the function's own block. look the list up on
    // the declaration rather than as the block's previous sibling, since a
    // return type can sit between the two
    const declaration = block.parent;
    if (declaration == null || declaration.name != "FunctionDeclaration")
        return;

    const parameters = declaration.getChild("ParameterList");
    if (parameters == null) return;

    checkForParametersRecursive(parameters.firstChild!, context, symbols);
};

const checkForParametersRecursive = (
    param: SyntaxNode,
    context: CompletionContext,
    symbols: Symbol[],
) => {
    if (param.name == "Parameter") {
        const identifierNode = param.getChild("Identifier");
        const identifier = context.state.sliceDoc(
            identifierNode?.from,
            identifierNode?.to,
        );

        const symbol: Symbol = {
            name: identifier,
            kind: "variable",
            type: getTypeName(context.state.doc, param.getChild("Type")),
        };
        symbols.push(symbol);
    }

    if (param.nextSibling == null) return;

    checkForParametersRecursive(param.nextSibling, context, symbols);
};

const applyIfCompletion = (
    view: EditorView,
    _completion: Completion,
    from: number,
    to: number,
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4) - 4;
    const indent = " ".repeat(indents);

    const insertion = `if  then\n\n${indent}end`;
    const newPos = from + 3;

    const transaction: TransactionSpec = {
        changes: { from: from, to: to, insert: insertion },
        selection: { anchor: newPos },
    };

    view.dispatch(transaction);
};

const applyFunctionCompletion = (
    view: EditorView,
    _completion: Completion,
    from: number,
    to: number,
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4);
    const indent = " ".repeat(indents);

    const insertion = `function ()\n\n${indent}end`;
    const newPos = from + 9;

    const transaction: TransactionSpec = {
        changes: { from: from, to: to, insert: insertion },
        selection: { anchor: newPos },
    };

    view.dispatch(transaction);
};

const applyForCompletion = (
    view: EditorView,
    _completion: Completion,
    from: number,
    to: number,
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4);
    const indent = " ".repeat(indents);

    const insertion = `loop  from  to\n\n${indent}end`;
    const newPos = from + 5;

    const transaction: TransactionSpec = {
        changes: { from: from, to: to, insert: insertion },
        selection: { anchor: newPos },
    };

    view.dispatch(transaction);
};

const applyUntilCompletion = (
    view: EditorView,
    _completion: Completion,
    from: number,
    to: number,
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4);
    const indent = " ".repeat(indents);

    const insertion = `loop until \n\n${indent}end`;
    const newPos = from + 11;

    const transaction: TransactionSpec = {
        changes: { from: from, to: to, insert: insertion },
        selection: { anchor: newPos },
    };

    view.dispatch(transaction);
};

const applyWhileCompletion = (
    view: EditorView,
    _completion: Completion,
    from: number,
    to: number,
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4);
    const indent = " ".repeat(indents);

    const insertion = `loop while \n\n${indent}end`;
    const newPos = from + 11;

    const transaction: TransactionSpec = {
        changes: { from: from, to: to, insert: insertion },
        selection: { anchor: newPos },
    };

    view.dispatch(transaction);
};

const ibCompletions = (context: CompletionContext) => {
    const word = context.matchBefore(/\w*/);
    if (word?.from == word?.to && !context.explicit) return null;

    const tree = syntaxTree(context.state);
    const symbols = resolveSymbols(tree, context, word?.text);

    const symbolOptions = symbols.map((symbol) => {
        return { label: symbol.name, type: symbol.kind };
    });

    return {
        from: word?.from,
        options: [
            {
                label: "if ... then",
                apply: applyIfCompletion,
                type: "keyword",
            },
            { label: "then", type: "keyword" },
            { label: "end", type: "keyword" },
            { label: "else", type: "keyword" },
            { label: "output", type: "keyword" },
            {
                label: "function ...()",
                apply: applyFunctionCompletion,
                type: "keyword",
            },
            { label: "return", type: "keyword" },
            { label: "new", type: "keyword" },
            { label: "not", type: "keyword" },
            { label: "and", type: "keyword" },
            { label: "or", type: "keyword" },
            { label: "mod", type: "keyword" },
            { label: "div", type: "keyword" },
            { label: "Void", type: "type" },
            { label: "Int", type: "type" },
            { label: "String", type: "type" },
            { label: "Boolean", type: "type" },
            { label: "Array", type: "type" },
            { label: "Collection", type: "type" },
            { label: "Stack", type: "type" },
            { label: "Queue", type: "type" },
            {
                label: "loop ... from ... to",
                apply: applyForCompletion,
                type: "keyword",
            },
            {
                label: "loop while",
                apply: applyWhileCompletion,
                type: "keyword",
            },
            {
                label: "loop until",
                apply: applyUntilCompletion,
                type: "keyword",
            },
            { label: "until", type: "keyword" },
            ...symbolOptions,
        ],
    };
};

export default ibCompletions;
