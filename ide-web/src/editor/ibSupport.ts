import { parser } from './parser'
import { LRLanguage, LanguageSupport, foldNodeProp, foldInside, indentNodeProp } from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'
import ibCompletions from './autocomplete';
import { completeFromList } from '@codemirror/autocomplete';

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
        autocomplete: ibCompletions(),
        commentTokens: { line: '#' },
    }
});

const ib = () => {
    console.log(LANG_DEF.data)
    const support = new LanguageSupport(LANG_DEF);
    return support;
};

export default ib;