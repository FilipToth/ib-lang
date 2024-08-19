import CodeMirror, { Prec, keymap } from '@uiw/react-codemirror'
import { coolGlow } from 'thememirror'
import ib from './ibSupport';
import { indentLess, indentMore, indentWithTab } from '@codemirror/commands';
import { acceptCompletion, completionStatus } from '@codemirror/autocomplete';

const Editor = () => {
    const ibSupport = ib();

    const keys = keymap.of([{
        key: 'Tab',
        run: (e) => {
            if (!completionStatus(e.state))
                return indentMore(e);

            return acceptCompletion(e);
        }
    }])

    const keyExtension = Prec.highest(keys);

    return (
        <CodeMirror
            height='100vh'
            theme={coolGlow}
            extensions={[ ibSupport, keyExtension ]}
        />
    );
};

export default Editor;