import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import ib from "./ibSupport";
import { indentLess, indentMore, indentWithTab } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import { useState } from "react";
import { TopBar } from "components/TopBar";

const Editor = () => {
    const [code, setCode] = useState("");
    const ibSupport = ib();

    const keys = keymap.of([
        {
            key: "Tab",
            run: (e) => {
                if (!completionStatus(e.state)) return indentMore(e);

                return acceptCompletion(e);
            },
        },
    ]);

    const keyExtension = Prec.highest(keys);
    return (
        <>
            <TopBar />
            <div>
                <OutputBar code={code} />
                <CodeMirror
                    height="100vh"
                    width="90vw"
                    theme={coolGlow}
                    extensions={[
                        ibSupport,
                        keyExtension,
                        indentUnit.of("    "),
                    ]}
                    value={code}
                    onChange={(value: string, _viewUpdate: ViewUpdate) => {
                        setCode(value);
                    }}
                />
            </div>
        </>
    );
};

export default Editor;
