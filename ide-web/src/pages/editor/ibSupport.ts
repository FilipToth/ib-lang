import { parser } from "./parser";
import {
    LRLanguage,
    LanguageSupport,
    foldNodeProp,
    foldInside,
    syntaxTree,
    indentService,
} from "@codemirror/language";
import { styleTags, tags as t } from "@lezer/highlight";
import ibCompletions from "./autocomplete";
import ibLinter from "./lint";
import { SyntaxNode, Tree } from "@lezer/common";

const LANG_DEF = LRLanguage.define({
    parser: parser.configure({
        props: [
            styleTags({
                "CallExpression/Identifier": t.function(t.variableName),
                "FunctionDeclaration/Identifier": t.function(t.variableName),
                Identifier: t.variableName,

                Boolean: t.bool,
                TypeAnnotation: t.typeName,
                String: t.string,
                Number: t.number,
                MiscOperator: t.operator,
                AdditiveOperator: t.operator,
                AssignmentOperator: t.operator,
                LineComment: t.lineComment,
                "( )": t.paren,
                "[ ]": t.squareBracket,

                IfKeyword: t.keyword,
                ThenKeyword: t.keyword,
                EndKeyword: t.keyword,
                ElseKeyword: t.keyword,
                OutputKeyword: t.keyword,
                FunctionKeyword: t.keyword,
                ReturnKeyword: t.keyword,
                NotKeyword: t.keyword,
                AndKeyword: t.operator,
                OrKeyword: t.operator,
                ModKeyword: t.operator,
                DivKeyword: t.operator,
                LoopKeyword: t.keyword,
                ForKeyword: t.keyword,
                FromKeyword: t.keyword,
                ToKeyword: t.keyword,
                WhileKeyword: t.keyword,
                UntilKeyword: t.keyword,
                NewKeyword: t.keyword,
            }),
            foldNodeProp.add({
                Application: foldInside,
            }),
        ],
    }),
    languageData: {
        commentTokens: { line: "#" },
    },
});

/// The statements whose body is indented a level. An else if chain is one
/// IfStatement, so its later branches sit level with the first.
const indentingNodes = new Set([
    "IfStatement",
    "FunctionDeclaration",
    "ForStatement",
    "WhileStatement",
    "UntilStatement",
]);

/// Whether `pos` is inside the body of `node`, which is what earns a level of
/// indentation.
const insideBody = (node: SyntaxNode, pos: number) => {
    if (!indentingNodes.has(node.name)) return false;

    // past the statement's `end`, its body is behind us: a line after a
    // finished if is no deeper than the if itself
    const end = node.getChild("EndKeyword");
    return end == null || pos <= end.from;
};

export const getIndent = (tree: Tree, pos: number, unit: number) => {
    // the statement being typed has nothing after the cursor to hold it
    // together yet, so what encloses the cursor is found from the text before
    // it. Looking forward instead finds only the whole program, which is why
    // a line after `if a then` at the end of the file was not indented.
    let node: SyntaxNode | null = tree.resolveInner(pos, -1);
    let indent = 0;

    while (node != null) {
        if (insideBody(node, pos)) indent += unit;

        node = node.parent;
    }

    return indent;
};

const indent = () => {
    return indentService.of((context, pos) => {
        const tree = syntaxTree(context.state);
        const indents = getIndent(tree, pos, context.unit);
        return indents;
    });
};

export const ib = () => {
    const ibAutocomplete = LANG_DEF.data.of({
        autocomplete: ibCompletions,
    });

    const indentExtension = indent();
    const support = new LanguageSupport(LANG_DEF, [
        indentExtension,
        ibAutocomplete,
        ibLinter,
    ]);
    return support;
};
