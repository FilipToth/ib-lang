import CodeMirror from '@uiw/react-codemirror'
import { coolGlow } from 'thememirror'
import ib from './ibSupport';

const Editor = () => {
    const ibSupport = ib();

    return (
        <CodeMirror
            height='100vh'
            theme={coolGlow}
            extensions={[ ibSupport ]}
        />
    );
};

export default Editor;