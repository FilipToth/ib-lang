import "assets/output-bar.css";
import { FunctionComponent, useRef } from "react";
import { runCode } from "services/server";

interface OutputProps {
    code: string;
}

const OutputBar: FunctionComponent<OutputProps> = ({ code }) => {
    const textAreaRef = useRef<HTMLTextAreaElement | null>(null);

    const onClick = async () => {
        textAreaRef.current!.value = "";
        const output = await runCode(code);
        textAreaRef.current!.value = output;
    };

    return (
        <div className="output-bar">
            <button className="run-button" onClick={onClick}>
                Run
            </button>
            <textarea className="text-area" ref={textAreaRef} />
        </div>
    );
};

export default OutputBar;
