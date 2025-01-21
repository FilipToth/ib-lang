import { Button, Stack, TextField, Typography } from "@mui/material";
import { FunctionComponent, useRef, useState } from "react";
import { runCode } from "services/server";

interface OutputProps {
    code: string;
}

const OutputBar: FunctionComponent<OutputProps> = ({ code }) => {
    const [output, setOutput] = useState("");

    const onClick = async () => {
        setOutput("");
        const codeOutput = await runCode(code);
        setOutput(codeOutput);
    };

    return (
        <Stack>
            <Button onClick={onClick}>Run</Button>
            <TextField
                multiline
                fullWidth
                value={output}
                slotProps={{
                    input: {
                        readOnly: true,
                    },
                }}
                sx={{
                    flex: 1,
                    "& .MuiInputBase-root": {
                        height: "100%",
                        alignItems: "start", // Aligns the text to the top
                    },
                }}
            />
            <Typography>Awaiting User Input</Typography>
            <Stack direction={"row"}>
                <TextField
                    multiline
                />
                <Button>Send</Button>
            </Stack>
        </Stack>
    );
};

export default OutputBar;
