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
import logTree from "./logTree";

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

const indent = () => {
    return indentService.of((context, pos) => {
        const tree = syntaxTree(context.state);
        const selectedNode = tree.resolveInner(pos, 1);
        logTree(tree.topNode);

        let node = selectedNode;
        let indent = 0;

        while (node.parent) {
            console.log(node.name);
            if (node.name == "IfStatement") {
                indent += context.unit;
            } else if (node.name == "FunctionDeclaration") {
                indent += context.unit;
            }

            node = node.parent;
        }

        return indent;
    });
};

const ib = () => {
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

export default ib;
