import { Text } from "@codemirror/text";
import { SyntaxNode } from "@lezer/common";

interface Node {
    name: string;
    text: string;
    children: Array<Node>;
}

const textify = (node: SyntaxNode, doc: Text) => {
    return doc.sliceString(node.from, node.to);
};

const logTreeInternal = (
    syntaxNode: SyntaxNode,
    indent: number,
    prev: Node,
    doc: Text
) => {
    const text = textify(syntaxNode, doc);
    const node: Node = { name: syntaxNode.name, children: [], text: text };

    if (syntaxNode.nextSibling != null)
        logTreeInternal(syntaxNode.nextSibling, indent + 4, prev, doc);

    if (syntaxNode.firstChild != null)
        logTreeInternal(syntaxNode.firstChild, indent + 4, node, doc);

    prev.children.push(node);
};

const logTree = (root: SyntaxNode, doc: Text) => {
    const node: Node = { name: "Root", children: [], text: textify(root, doc) };
    logTreeInternal(root, 0, node, doc);

    const msg = JSON.stringify(node, null, 4);
    console.log(msg);
};

export default logTree;
