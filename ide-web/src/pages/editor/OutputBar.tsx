import {
    Alert,
    Button,
    CircularProgress,
    Snackbar,
    Stack,
    TextField,
    Typography,
} from "@mui/material";
import { PlayArrowRounded } from "@mui/icons-material";
import { FunctionComponent, useEffect, useRef, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { auth } from "services/firebase";
import { RuntimeErrorRange } from "./runtimeError";
import OutputView from "./OutputView";

const WS_URL = process.env.REACT_APP_WEBSOCKETS_URL;

interface OutputProps {
    code: string;
    fileId: string | undefined;
    filename: string | undefined;
    /// Reports where a runtime error happened so the editor can highlight it,
    /// and null once a new run starts.
    onRuntimeError: (error: RuntimeErrorRange | null) => void;
}

enum WebSocketMessageKind {
    Execute,
    Output,
    Input,
    RuntimeError,
    AnalysisError,
}

interface WebSocketMessage {
    kind: WebSocketMessageKind;
    payload: string;
    file_id?: string;
    offset_start?: number;
    offset_end?: number;
}

const OutputBar: FunctionComponent<OutputProps> = ({
    code,
    fileId,
    filename,
    onRuntimeError,
}) => {
    const [output, setOutput] = useState("");
    /// The file the output came from. The panel stays put when another tab is
    /// opened, so without it the output could pass for that file's.
    const [ranFile, setRanFile] = useState<string | null>(null);
    const [awaitingInput, setAwaitingInput] = useState(false);
    const [input, setInput] = useState("");
    const [running, setRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [sockerUrl, setSocketUrl] = useState<string | null>(null);
    const { sendMessage, lastMessage, readyState } = useWebSocket(sockerUrl);

    const showError = (msg: string) => {
        setError(msg);
        setTimeout(() => setError(null), 4000);
    };

    const onClick = async () => {
        setOutput("");
        onRuntimeError(null);
        if (WS_URL == undefined) {
            console.error("Wrong env config, websockets url is undefined");
            return;
        }

        if (fileId == undefined) {
            showError("Open a file before running.");
            return;
        }

        // the socket is authenticated with the same Firebase ID token the REST
        // API uses; it goes in the query string because the browser WebSocket
        // API cannot set request headers
        const jwt = await auth.currentUser?.getIdToken();
        if (jwt == undefined) {
            showError("You are signed out. Sign in again to run code.");
            return;
        }

        setRanFile(filename ?? null);
        setSocketUrl(`${WS_URL}?token=${encodeURIComponent(jwt)}`);
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

                // the snackbar goes away on its own, the highlight stays until
                // the next run or the next edit
                if (
                    msg.offset_start != undefined &&
                    msg.offset_end != undefined
                ) {
                    onRuntimeError({
                        start: msg.offset_start,
                        end: msg.offset_end,
                    });
                }

                break;
            case WebSocketMessageKind.AnalysisError:
                // the program was not run at all, and the linter already
                // underlines why
                showError("Cannot run: " + msg.payload);
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
                    file_id: fileId,
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
            <Stack flex={1} minWidth={0} p={1} gap={1}>
                <Stack
                    direction={"row"}
                    alignItems={"center"}
                    justifyContent={"space-between"}
                >
                    <Typography
                        variant="subtitle1"
                        fontWeight={600}
                        noWrap
                        title={ranFile ?? undefined}
                        sx={{ minWidth: 0 }}
                    >
                        {ranFile == null ? "Output" : `Output: ${ranFile}`}
                    </Typography>
                    <Button
                        variant="contained"
                        onClick={onClick}
                        disabled={running}
                        // a long file name is cut short instead
                        sx={{ flexShrink: 0 }}
                        startIcon={
                            running ? (
                                <CircularProgress size={16} color="inherit" />
                            ) : (
                                <PlayArrowRounded />
                            )
                        }
                    >
                        Run
                    </Button>
                </Stack>
                <OutputView output={output} />
                {awaitingInput && (
                    <Typography variant="body2" color="primary">
                        Waiting for input
                    </Typography>
                )}
                <Stack direction={"row"} gap={1} alignItems={"flex-start"}>
                    <TextField
                        multiline
                        // grows with what is typed, up to a point, then
                        // scrolls instead of eating the output's space
                        maxRows={4}
                        fullWidth
                        size="small"
                        placeholder="Input"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                    />
                    <Button
                        variant="outlined"
                        disabled={!awaitingInput}
                        onClick={sendInput}
                        sx={{ height: 40 }}
                    >
                        Send
                    </Button>
                </Stack>
            </Stack>
        </>
    );
};

export default OutputBar;
