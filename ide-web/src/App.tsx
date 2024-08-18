import './App.css'
import { MonacoEditorLanguageClientWrapper } from "monaco-editor-wrapper";
import { useEffect, useRef } from "react";
import { helloConfig } from "./config";

// https://medium.com/@cjayashantha/creating-a-web-editor-using-angular-and-monaco-for-a-custom-language-server-part-1-4424d9be9c7b

const App = () => {
    const editorRef = useRef(null);
    const wrapper = new MonacoEditorLanguageClientWrapper();

    useEffect(() => {
        const setup = async () => {
            await wrapper.dispose();
            await wrapper.initAndStart(helloConfig, editorRef.current);
        };

        console.log('f');
        setup();
    }, []);

    return (
        <div className="editor" id="editor" ref={editorRef}></div>
    );
};

export default App;