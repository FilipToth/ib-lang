import { Button, Stack, TextField, Typography } from "@mui/material";
import { FunctionComponent, useEffect, useRef, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

const WS_URL = process.env.REACT_APP_WEBSOCKETS_URL;

interface OutputProps {
    code: string;
}

enum WebSocketMessageKind {
    Execute,
    Output,
    Input,
}

interface WebSocketMessage {
    kind: WebSocketMessageKind;
    payload: string;
}

const OutputBar: FunctionComponent<OutputProps> = ({ code }) => {
    const [output, setOutput] = useState("");

    const [sockerUrl, setSocketUrl] = useState<string | null>(null);
    const { sendMessage, lastMessage, readyState } = useWebSocket(sockerUrl);

    const onClick = async () => {
        setOutput("");
        if (WS_URL == undefined) {
            console.error("Wrong env config, websockets url is undefined");
            return;
        }

        setSocketUrl(WS_URL);
    };

    useEffect(() => {
        // new message
        if (lastMessage == null) return;

        const msg: WebSocketMessage = JSON.parse(lastMessage.data);
        switch (msg.kind) {
            case WebSocketMessageKind.Execute:
                // server not supposed to send execute requests
                break;
            case WebSocketMessageKind.Input:
                // not implemented
                break;
            case WebSocketMessageKind.Output:
                setOutput("Hellou");
                break;
        }
    }, [lastMessage]);

    useEffect(() => {
        // socket state changed
        switch (readyState) {
            case ReadyState.OPEN:
                // send execute request
                const msg: WebSocketMessage = {
                    kind: WebSocketMessageKind.Execute,
                    payload: code,
                };

                const msg_raw = JSON.stringify(msg);
                sendMessage(msg_raw);
                break;
            case ReadyState.CLOSING:
            case ReadyState.CLOSED:
                setSocketUrl(null);
                break;
        }
    }, [readyState]);

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
                <TextField multiline />
                <Button>Send</Button>
            </Stack>
        </Stack>
    );
};

export default OutputBar;
