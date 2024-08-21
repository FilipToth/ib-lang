import { CompletionContext, completeFromList } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { SyntaxNode, Tree } from "@lezer/common";
import { EditorState } from "@uiw/react-codemirror";
import logTree from "./logTree";

interface Symbol {
    name: string,
    type: string
}

const resolveSymbols = (tree: Tree, context: CompletionContext, word: string | undefined) => {
    const nodeBefore = tree.resolveInner(context.pos, -1);

    logTree(tree.topNode, context);

    const scopes: SyntaxNode[] = [];
    getScopesRecursive(nodeBefore, scopes);

    const symbols: Symbol[] = [];
    for (const scope of scopes) {
        if (scope.firstChild == null)
            continue;

        resolveSymbolsInScope(scope.firstChild, context, symbols);
    }

    const resolvedSymbols: Symbol[] = [];
    symbols.forEach((symbol) => {
        if (symbol.name == word)
            return;

        resolvedSymbols.push(symbol);
    });

    return resolvedSymbols;
};

const getScopesRecursive = (node: SyntaxNode, scopes: SyntaxNode[]) => {
    if (node.name == "Block")
        scopes.push(node);

    if (node.parent == null)
        return;

    getScopesRecursive(node.parent, scopes);
};

const resolveSymbolsInScope = (scopeChild: SyntaxNode, context: CompletionContext, symbols: Symbol[]) => {
    // a scope will always contain atoms, the first
    // child of the atom is the actual node
    let identifier = null;
    let kind = null;

    const node = scopeChild.firstChild!;
    if (node.name == "VariableAssignment") {
        const identifierNode = node.getChild("Identifier");
        identifier = context.state.sliceDoc(identifierNode?.from, identifierNode?.to);

        kind = "variable";
    } else if (node.name == "FunctionDeclaration") {
        const identifierNode = node.getChild("Identifier");
        identifier = context.state.sliceDoc(identifierNode?.from, identifierNode?.to);

        kind = "function";
    }

    if (identifier != null && kind != null) {
        symbols.push({ name: identifier, type: kind });
    }

    if (scopeChild.nextSibling == null)
        return;

    resolveSymbolsInScope(scopeChild.nextSibling, context, symbols);
};

const ibCompletions = (context: CompletionContext) => {
    let word = context.matchBefore(/\w*/)
    if (word?.from == word?.to && !context.explicit)
        return null;

    const tree = syntaxTree(context.state);
    const symbols = resolveSymbols(tree, context, word?.text);

    const symbolOptions = symbols.map((symbol) => {
        return { label: symbol.name, type: symbol.type };
    });

    return {
        from: word?.from,
        options: [
            {label: "if", type: "keyword"},
            {label: "then", type: "keyword"},
            {label: "end", type: "keyword"},
            {label: "else", type: "keyword"},
            {label: "output", type: "keyword"},
            {label: "function", type: "keyword"},
            {label: "return", type: "keyword"},
            {label: "not", type: "keyword"},
            {label: "Void", type: "type"},
            {label: "Int", type: "type"},
            {label: "String", type: "type"},
            {label: "Boolean", type: "type"},
            ...symbolOptions
        ]
    }
};

export default ibCompletions;