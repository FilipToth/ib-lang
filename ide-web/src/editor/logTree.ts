import { CompletionContext } from "@codemirror/autocomplete";
import { SyntaxNode } from "@lezer/common";

interface Node {
    name: string,
    children: Array<Node>
}

const logTreeInternal = (syntaxNode: SyntaxNode, context: CompletionContext, indent: number, prev: Node) => {
    const t = context.state.sliceDoc(syntaxNode.from, syntaxNode.to);
    const node: Node = { name: syntaxNode.name, children: [] };

    if (syntaxNode.nextSibling != null)
        logTreeInternal(syntaxNode.nextSibling, context, indent + 4, prev);

    if (syntaxNode.firstChild != null)
        logTreeInternal(syntaxNode.firstChild, context, indent + 4, node);

    prev.children.push(node);
}

const logTree = (root: SyntaxNode, context: CompletionContext) => {
    const node: Node = { name: "Root", children: [] };
    logTreeInternal(root, context, 0, node);

    const msg = JSON.stringify(node, null, 4);
    console.log(msg);
};

/* [
    { name: "test", children: [] }
] */

export default logTree;