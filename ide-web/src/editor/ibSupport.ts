import { parser } from './parser'
import { LRLanguage, LanguageSupport, foldNodeProp, foldInside, indentNodeProp } from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'
import ibCompletions from './autocomplete';

const LANG_DEF = LRLanguage.define({
    parser: parser.configure({
        props: [
            styleTags({
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
                Application: (context) => context.column(context.node.from)
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