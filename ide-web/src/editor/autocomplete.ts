import { CompletionContext, completeFromList } from "@codemirror/autocomplete";

const ibCompletions = () => {
    return completeFromList([
        { label: 'if', type: 'keyword' },
        { label: 'then', type: 'keyword' },
        { label: 'end', type: 'keyword' },
        { label: 'else', type: 'keyword' },
        { label: 'output', type: 'keyword' },
        { label: 'function', type: 'keyword' },
        { label: 'return', type: 'keyword' },
        { label: 'not', type: 'keyword' },
        { label: 'Void', type: 'type' },
        { label: 'Int', type: 'type' },
        { label: 'String', type: 'type' },
        { label: 'Boolean', type: 'type' },
    ])
};

export default ibCompletions;