import "assets/output-bar.css"
import axios from "axios";
import { FunctionComponent, useRef } from "react";

interface OutputProps {
    code: string
}

const OutputBar: FunctionComponent<OutputProps> = ({ code }) => {
    const textAreaRef = useRef<HTMLTextAreaElement | null>(null);

    const onClick = () => {
        textAreaRef.current!.value = "";
        runCode(code, textAreaRef.current!);
    }

    return (
        <div className="output-bar">
            <button className="run-button" onClick={onClick}>Run</button>
            <textarea className="text-area" ref={textAreaRef} />
        </div>
    );
};

const runCode = (code: String, textArea: HTMLTextAreaElement) => {
    const req = axios.post('http://127.0.0.1:8080', code);
    req.then((resp) => {
        const data = resp.data;
        const output = data.output;
        textArea.value = output;
    });
}

export default OutputBar;