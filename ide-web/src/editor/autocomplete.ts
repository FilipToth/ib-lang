import { CompletionContext, completeFromList } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { SyntaxNode } from "@lezer/common";
import { EditorState } from "@uiw/react-codemirror";

interface Symbol {
    name: string,
    type: string
}

const resolveSymbols = (root: SyntaxNode, state: EditorState) => {
    const symbols: Symbol[] = [];

    console.log(root.lastChild);
    root.getChildren("Atom").forEach((node) => {
        const token = node.firstChild;
        if (token?.name != "Identifier")
            return;

        const text = state.sliceDoc(token.from, token.to);
        symbols.push({ name: text, type: "variable" });
    });

    return symbols;
};

const ibCompletions = (context: CompletionContext) => {
    let word = context.matchBefore(/\w*/)
    if (word?.from == word?.to && !context.explicit)
        return null;

    const tree = syntaxTree(context.state);
    const symbols = resolveSymbols(tree.topNode, context.state);

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