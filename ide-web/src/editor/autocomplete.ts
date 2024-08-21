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
    const root = tree.topNode;
    const nodeBefore = tree.resolveInner(context.pos, -1);

    logTree(root, context);

    const symbols: Symbol[] = [];
    root.getChildren("Atom").forEach((node) => {
        const token = node.firstChild;
        if (token?.name != "Identifier")
            return;

        const text = context.state.sliceDoc(token.from, token.to);
        if (text == word)
            return;

        symbols.push({ name: text, type: "variable" });
    });

    return symbols;
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