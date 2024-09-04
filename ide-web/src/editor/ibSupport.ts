import { parser } from './parser'
import { LRLanguage, LanguageSupport, foldNodeProp, foldInside, indentNodeProp } from '@codemirror/language'
import { Tag, styleTags, tags as t } from '@lezer/highlight'
import ibCompletions from './autocomplete';

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
            indentNodeProp.add({
                Application: (context) => context.column(context.node.from),
                Block: (context) => context.column(context.node.from) + context.unit
            }),
            foldNodeProp.add({
                Application: foldInside
            }),
        ]
    }),
    languageData: {
        commentTokens: { line: '#' },
    }
});

const ib = () => {
    const ibAutocomplete = LANG_DEF.data.of({
        autocomplete: ibCompletions
    });

    const support = new LanguageSupport(LANG_DEF, [ibAutocomplete]);
    return support;
};

export default ib;