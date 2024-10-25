import { SyntaxNode } from "@lezer/common";

interface Node {
    name: string,
    children: Array<Node>
}

const logTreeInternal = (syntaxNode: SyntaxNode, indent: number, prev: Node) => {
    const node: Node = { name: syntaxNode.name, children: [] };

    if (syntaxNode.nextSibling != null)
        logTreeInternal(syntaxNode.nextSibling, indent + 4, prev);

    if (syntaxNode.firstChild != null)
        logTreeInternal(syntaxNode.firstChild, indent + 4, node);

    prev.children.push(node);
}

const logTree = (root: SyntaxNode) => {
    const node: Node = { name: "Root", children: [] };
    logTreeInternal(root, 0, node);

    const msg = JSON.stringify(node, null, 4);
    console.log(msg);
};

export default logTree;