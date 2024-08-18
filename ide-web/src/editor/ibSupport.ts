import { parser } from './parser'
import { LRLanguage, LanguageSupport, foldNodeProp, foldInside, indentNodeProp } from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'

const LANG_DEF = LRLanguage.define({
    parser: parser.configure({
        props: [
            styleTags({
                Identifier: t.variableName,
                Boolean: t.bool,
                Keyword: t.keyword,
                TypeAnnotation: t.typeName,
                String: t.string,
                Number: t.number,
                Operator: t.operator,
                LineComment: t.lineComment,
                "( )": t.paren
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
        commentTokens: { line: '#' }
    }
});

const ib = () => {
    return new LanguageSupport(LANG_DEF)
};

export default ib;