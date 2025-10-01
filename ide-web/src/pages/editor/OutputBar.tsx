import {
    Alert,
    Button,
    CircularProgress,
    Snackbar,
    Stack,
    TextField,
    Typography,
} from "@mui/material";
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
    RuntimeError,
}

interface WebSocketMessage {
    kind: WebSocketMessageKind;
    payload: string;
}

const OutputBar: FunctionComponent<OutputProps> = ({ code }) => {
    const [output, setOutput] = useState("");
    const [awaitingInput, setAwaitingInput] = useState(false);
    const [input, setInput] = useState("");
    const [running, setRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);

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
                setAwaitingInput(true);
                break;
            case WebSocketMessageKind.Output:
                setOutput((val) => (val += msg.payload));
                break;
            case WebSocketMessageKind.RuntimeError:
                // display error
                setError("Runtime Error: " + msg.payload);
                setTimeout(() => {
                    setError(null);
                }, 4000);

                break;
        }
    }, [lastMessage]);

    useEffect(() => {
        // socket state changed
        switch (readyState) {
            case ReadyState.OPEN:
                // send execute request
                setRunning(true);
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
                setRunning(false);
                break;
        }
    }, [readyState]);

    const sendInput = () => {
        const msg: WebSocketMessage = {
            kind: WebSocketMessageKind.Input,
            payload: input,
        };

        const msg_raw = JSON.stringify(msg);
        sendMessage(msg_raw);
        setAwaitingInput(false);
        setInput("");
    };

    return (
        <>
            <Snackbar
                anchorOrigin={{ vertical: "top", horizontal: "center" }}
                open={error != null}
            >
                <Alert severity="error" variant="filled">
                    {error}
                </Alert>
            </Snackbar>
            <Stack>
                <Stack direction={"row"}>
                    <Button
                        onClick={onClick}
                        fullWidth
                        sx={{
                            gap: "10%",
                        }}
                        disabled={running}
                    >
                        <Typography>Run</Typography>
                        {running && <CircularProgress size={20} />}
                    </Button>
                </Stack>
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
                            alignItems: "start",
                        },
                    }}
                />
                {awaitingInput && <Typography>Awaiting User Input</Typography>}
                <Stack direction={"row"}>
                    <TextField
                        multiline
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                    />
                    <Button disabled={!awaitingInput} onClick={sendInput}>
                        Send
                    </Button>
                </Stack>
            </Stack>
        </>
    );
};

export default OutputBar;
