import {
    Alert,
    Button,
    IconButton,
    Snackbar,
    Stack,
    TextField,
    Tooltip,
    Typography,
} from "@mui/material";
import { ClearAll, PlayArrowRounded, StopRounded } from "@mui/icons-material";
import { FunctionComponent, useEffect, useRef, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { auth } from "services/firebase";
import { RuntimeErrorRange } from "./runtimeError";
import OutputView, {
    CodeLocation,
    OutputEntry,
    appendEntry,
} from "./OutputView";

const WS_URL = process.env.REACT_APP_WEBSOCKETS_URL;

/// Marks the Firebase ID token that follows it in the subprotocol list. The
/// server answers with this same name, and refuses the handshake without it.
const AUTH_SUBPROTOCOL = "ib-auth-v1";

interface OutputProps {
    code: string;
    fileId: string | undefined;
    filename: string | undefined;
    /// Reports where a runtime error happened so the editor can highlight it,
    /// and null once a new run starts.
    onRuntimeError: (error: RuntimeErrorRange | null) => void;
    /// Asks the editor to show the place a clicked error came from.
    goTo: (at: CodeLocation) => void;
}

enum WebSocketMessageKind {
    Execute,
    Output,
    Input,
    RuntimeError,
    AnalysisError,
    /// Asks the server to stop the running program, and comes back when it has.
    Stop,
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
    goTo,
}) => {
    const [entries, setEntries] = useState<OutputEntry[]>([]);
    /// The file the output came from. The panel stays put when another tab is
    /// opened, so without it the output could pass for that file's.
    const [ranFile, setRanFile] = useState<string | null>(null);
    const [awaitingInput, setAwaitingInput] = useState(false);
    const [input, setInput] = useState("");
    const [running, setRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [sockerUrl, setSocketUrl] = useState<string | null>(null);
    /// The token the socket authenticates with. It is held only while a run is
    /// live, and travels as a subprotocol rather than in the URL, which would
    /// put it in access logs and browser history.
    const [jwt, setJwt] = useState<string | null>(null);

    const { sendMessage, lastMessage, readyState } = useWebSocket(sockerUrl, {
        protocols: jwt == null ? undefined : [AUTH_SUBPROTOCOL, jwt],
    });

    /// Set from asking for a run until the socket opens. A close while it is
    /// set means the handshake was refused -- an expired token, a server with
    /// no room, or a tab still running the code it loaded before a deploy --
    /// which would otherwise show as nothing happening at all.
    const awaitingOpen = useRef(false);

    /// Puts a line in the output panel: what the program printed, or why it
    /// stopped.
    const append = (
        kind: OutputEntry["kind"],
        text: string,
        at?: CodeLocation,
    ) => {
        setEntries((es) => appendEntry(es, kind, text, at));
    };

    /// For problems that are not the program's: the snackbar clears itself,
    /// rather than on a timer of its own that a later error would outlive.
    const showError = (msg: string) => setError(msg);

    const onClick = async () => {
        setEntries([]);
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
        // API uses. the browser WebSocket API cannot set request headers, so it
        // travels as a subprotocol, which is the one part of the handshake the
        // browser does let us choose
        const token = await auth.currentUser?.getIdToken();
        if (token == undefined) {
            showError("You are signed out. Sign in again to run code.");
            return;
        }

        setRanFile(filename ?? null);
        awaitingOpen.current = true;

        // both land in one render, so the socket opens with the token set
        setJwt(token);
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
                append("output", msg.payload);
                break;
            case WebSocketMessageKind.RuntimeError: {
                const at =
                    msg.offset_start != undefined &&
                    msg.offset_end != undefined &&
                    fileId != undefined
                        ? {
                              fileId: fileId,
                              start: msg.offset_start,
                              end: msg.offset_end,
                          }
                        : undefined;

                // in the output, where it belongs to the run that caused it
                // and stays until the next one. It is clicked to go to the
                // line it came from
                append("error", `\nRuntime error: ${msg.payload}\n`, at);

                // the highlight stays until the next run or the next edit
                if (at != null) {
                    onRuntimeError({ start: at.start, end: at.end });
                }

                break;
            }
            case WebSocketMessageKind.Stop:
                // the run ended because it was asked to, which is worth saying:
                // the output would otherwise just stop part way
                append("error", `\n${msg.payload}\n`);
                break;
            case WebSocketMessageKind.AnalysisError:
                // the program was not run at all, and the linter already
                // underlines why
                append("error", `Cannot run: ${msg.payload}\n`);
                break;
        }
    }, [lastMessage]);

    useEffect(() => {
        // socket state changed
        switch (readyState) {
            // braced: what is declared here belongs to this case alone
            case ReadyState.OPEN: {
                // send execute request
                awaitingOpen.current = false;
                setRunning(true);
                const msg: WebSocketMessage = {
                    kind: WebSocketMessageKind.Execute,
                    payload: code,
                    file_id: fileId,
                };

                sendMessage(JSON.stringify(msg));
                break;
            }
            case ReadyState.CLOSING:
            case ReadyState.CLOSED:
                if (awaitingOpen.current) {
                    awaitingOpen.current = false;
                    showError(
                        "Could not start the run. Reload the page and try again.",
                    );
                }

                setSocketUrl(null);
                // a live token is not worth keeping between runs
                setJwt(null);
                setRunning(false);
                // the program is gone; nothing is waiting to be typed at
                setAwaitingInput(false);
                break;
        }
    }, [readyState]);

    /// Asks the server to stop the program. The server stops it between steps,
    /// says so, and closes, which is what clears `running`.
    ///
    /// The button stays live rather than waiting on that: if the socket is
    /// already gone, dropping it here is what ends the run, and pressing again
    /// only asks twice.
    const stop = () => {
        if (readyState != ReadyState.OPEN) {
            // asked for, so not a handshake that failed
            awaitingOpen.current = false;
            setSocketUrl(null);
            return;
        }

        const msg: WebSocketMessage = {
            kind: WebSocketMessageKind.Stop,
            payload: "",
        };

        sendMessage(JSON.stringify(msg));
    };

    const sendInput = () => {
        if (!awaitingInput) return;

        const msg: WebSocketMessage = {
            kind: WebSocketMessageKind.Input,
            payload: input,
        };

        sendMessage(JSON.stringify(msg));

        // in the output as well, so the run reads back as it happened
        append("input", `${input}\n`);

        setAwaitingInput(false);
        setInput("");
    };

    /// Empties the panel. The file it names goes with it, unless the run it
    /// belongs to is still going.
    const clearOutput = () => {
        setEntries([]);
        if (!running) setRanFile(null);
    };

    /// Ready to type the moment the program asks, without reaching for the
    /// mouse.
    const inputField = useRef<HTMLInputElement | null>(null);

    useEffect(() => {
        if (awaitingInput) inputField.current?.focus();
    }, [awaitingInput]);

    return (
        <>
            <Snackbar
                anchorOrigin={{ vertical: "top", horizontal: "center" }}
                open={error != null}
                autoHideDuration={4000}
                onClose={(_, reason) => {
                    if (reason != "clickaway") setError(null);
                }}
            >
                <Alert
                    severity="error"
                    variant="filled"
                    onClose={() => setError(null)}
                >
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
                    {/* a long file name is cut short instead of pushing
                        these out of the panel */}
                    <Stack
                        direction={"row"}
                        gap={1}
                        alignItems={"center"}
                        flexShrink={0}
                    >
                        <Tooltip title="Clear output">
                            <span>
                                <IconButton
                                    size="small"
                                    onClick={clearOutput}
                                    disabled={entries.length == 0}
                                    aria-label="Clear output"
                                >
                                    <ClearAll fontSize="small" />
                                </IconButton>
                            </span>
                        </Tooltip>
                        {/* one button: it runs the program, and while that
                            program is running it stops it */}
                        <Button
                            variant="contained"
                            color={running ? "error" : "primary"}
                            onClick={running ? stop : onClick}
                            aria-label={running ? "Stop program" : "Run program"}
                            startIcon={
                                running ? (
                                    <StopRounded />
                                ) : (
                                    <PlayArrowRounded />
                                )
                            }
                        >
                            {running ? "Stop" : "Run"}
                        </Button>
                    </Stack>
                </Stack>
                <OutputView entries={entries} goTo={goTo} />
                {/* only while the program is waiting: there is nothing to
                    type at otherwise */}
                {awaitingInput && (
                    <>
                        <Typography variant="body2" color="primary">
                            Waiting for input
                        </Typography>
                        <Stack
                            direction={"row"}
                            gap={1}
                            alignItems={"flex-start"}
                        >
                            <TextField
                                multiline
                                // grows with what is typed, up to a point,
                                // then scrolls instead of eating the
                                // output's space
                                maxRows={4}
                                fullWidth
                                autoFocus
                                inputRef={inputField}
                                size="small"
                                placeholder={
                                    "Enter to send, " +
                                    "shift-enter for a new line"
                                }
                                value={input}
                                onChange={(e) => setInput(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key != "Enter" || e.shiftKey) return;

                                    // a newline would otherwise go into the
                                    // input rather than sending it
                                    e.preventDefault();
                                    sendInput();
                                }}
                            />
                            <Button
                                variant="outlined"
                                onClick={sendInput}
                                sx={{ height: 40 }}
                            >
                                Send
                            </Button>
                        </Stack>
                    </>
                )}
            </Stack>
        </>
    );
};

export default OutputBar;
