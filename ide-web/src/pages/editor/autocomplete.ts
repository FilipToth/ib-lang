import { Completion, CompletionContext } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { SyntaxNode, Tree } from "@lezer/common";
import { EditorView, TransactionSpec } from "@uiw/react-codemirror";
import { getIndent } from "./ibSupport";
import { Text } from "@codemirror/text";

interface Symbol {
    name: string;
    kind: string;
    typeName: string | null;
}

const resolveSymbols = (
    tree: Tree,
    context: CompletionContext,
    word: string | undefined
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
    symbols: Symbol[]
) => {
    // a scope will always contain atoms, the first
    // child of the atom is the actual node
    const node = scopeChild.firstChild;
    if (node == null) return;

    const identifierNode = node.getChild("Identifier");
    const identifier = context.state.sliceDoc(
        identifierNode?.from,
        identifierNode?.to
    );

    const document = context.view?.state.doc!;

    let kind = null;
    let typeName = null;

    if (node.name == "VariableAssignment") {
        kind = "variable";
        typeName = getVariableDeclarationType(document, node);
    } else if (node.name == "FunctionDeclaration") {
        kind = "function";
    } else {
        return;
    }

    if (identifier != null && kind != null) {
        symbols.push({ name: identifier, kind: kind, typeName: typeName });
    }

    if (scopeChild.nextSibling == null) return;

    resolveSymbolsInScope(scopeChild.nextSibling, context, symbols);
};

const getVariableDeclarationType = (document: Text, varNode: SyntaxNode) => {
    const expr = varNode.getChild("Expression");
    if (expr == null) return null;

    const objInstantiation = expr.getChild("ObjectInstantiationExpression");
    if (objInstantiation == null) {
        console.log("Obj instamtiation is null");
        return null;
    }

    const typeNodes = objInstantiation.getChildren("TypeAnnotation");
    const typeNode = typeNodes[0];

    const typeName = document.sliceString(typeNode.from, typeNode.to);
    return typeName;
};

const checkForParameters = (
    block: SyntaxNode,
    context: CompletionContext,
    symbols: Symbol[]
) => {
    // the parameter list will always be the
    // prev sibling to the block in a function
    // declaration

    const prev = block.prevSibling;
    if (prev == null || prev.name != "ParameterList") return;

    checkForParametersRecursive(prev.firstChild!, context, symbols);
};

const checkForParametersRecursive = (
    param: SyntaxNode,
    context: CompletionContext,
    symbols: Symbol[]
) => {
    if (param.name == "Parameter") {
        const identifierNode = param.getChild("Identifier");
        const identifier = context.state.sliceDoc(
            identifierNode?.from,
            identifierNode?.to
        );

        const symbol: Symbol = {
            name: identifier,
            kind: "variable",
            typeName: null,
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
    to: number
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
    to: number
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
    to: number
) => {
    const tree = syntaxTree(view.state);
    const indents = getIndent(tree, from, 4);
    const indent = " ".repeat(indents);

    const insertion = `loop for  from  to\n\n${indent}end`;
    const newPos = from + 9;

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
    to: number
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
    let word = context.matchBefore(/\w*/);
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
            { label: "not", type: "keyword" },
            { label: "Void", type: "type" },
            { label: "Int", type: "type" },
            { label: "String", type: "type" },
            { label: "Boolean", type: "type" },
            {
                label: "loop for",
                apply: applyForCompletion,
                type: "keyword",
            },
            {
                label: "loop while",
                apply: applyWhileCompletion,
                type: "keyword",
            },
            ...symbolOptions,
        ],
    };
};

export default ibCompletions;
