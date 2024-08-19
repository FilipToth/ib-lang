import { CompletionContext, completeFromList } from "@codemirror/autocomplete";

const ibCompletions = (context: CompletionContext) => {
    let word = context.matchBefore(/\w*/)
    if (word?.from == word?.to && !context.explicit)
        return null

    console.log(word?.text);

    return {
        from: word?.from,
        options: [
            {label: "if", type: "keyword"},
            {label: "then", type: "keyword"},
            {label: "end", type: "keyword"},
            {label: "else", type: "keyword"},
            {label: "output", type: "keyword"},
            {label: "function", type: "keyword"},
            {label: "return", type: "keyword"},
            {label: "not", type: "keyword"},
            {label: "Void", type: "type"},
            {label: "Int", type: "type"},
            {label: "String", type: "type"},
            {label: "Boolean", type: "type"},
        ]
    }
};

export default ibCompletions;