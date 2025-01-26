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
import { Tree } from "@lezer/common";

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
                AssignmentOperator: t.operator,
                LineComment: t.lineComment,
                "( )": t.paren,

                IfKeyword: t.keyword,
                ThenKeyword: t.keyword,
                EndKeyword: t.keyword,
                ElseKeyword: t.keyword,
                OutputKeyword: t.keyword,
                FunctionKeyword: t.keyword,
                ReturnKeyword: t.keyword,
                NotKeyword: t.keyword,
                LoopKeyword: t.keyword,
                ForKeyword: t.keyword,
                FromKeyword: t.keyword,
                ToKeyword: t.keyword,
                WhileKeyword: t.keyword,
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

export const getIndent = (tree: Tree, pos: number, unit: number) => {
    const selectedNode = tree.resolveInner(pos, 1);

    let node = selectedNode;
    let indent = 0;

    while (node.parent) {
        console.log(node.name);
        if (node.name == "IfStatement") {
            indent += unit;
        } else if (node.name == "FunctionDeclaration") {
            indent += unit;
        }

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
