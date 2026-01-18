import { Alert, Box, CircularProgress, IconButton, Stack } from "@mui/material";
import {
    AddRounded,
    CenterFocusStrong,
    RefreshRounded,
    RemoveRounded,
} from "@mui/icons-material";
import { useCallback, useEffect, useRef, useState } from "react";
import { TransformComponent, TransformWrapper } from "react-zoom-pan-pinch";
import { getControlFlowGraph } from "services/server";

/// How long edits have to pause before the graph is drawn again. Drawing
/// means a round trip and a graphviz layout, so it waits for a real pause
/// rather than following every keystroke.
const redrawDelay = 800;

/// Draws the control flow graph of `code`.
///
/// The server analyzes the source and returns Graphviz DOT; the layout is done
/// here by Graphviz itself, compiled to WebAssembly. That module is a couple of
/// megabytes, so it is imported when a graph is first opened rather than
/// bundled into the initial load.
const GraphView = ({ code }: { code: string }) => {
    const container = useRef<HTMLDivElement | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    /// Whether a drawing is on screen, which a failed redraw leaves in place.
    const [drawn, setDrawn] = useState(false);

    // the code the drawing is of: the code as given, once edits pause
    const [source, setSource] = useState(code);

    useEffect(() => {
        if (code == source) return;

        const timer = setTimeout(() => setSource(code), redrawDelay);
        return () => clearTimeout(timer);
    }, [code, source]);

    const draw = useCallback(async (): Promise<() => void> => {
        let cancelled = false;

        setLoading(true);
        setError(null);

        try {
            const graph = await getControlFlowGraph(source);

            if (graph.dot == null) {
                const first = graph.diagnostics[0];
                const reason =
                    first == null
                        ? "the program does not compile"
                        : first.message;

                if (!cancelled) {
                    setError(reason);
                    setLoading(false);
                }

                return () => {
                    cancelled = true;
                };
            }

            const { instance } = await import("@viz-js/viz");
            const viz = await instance();
            const svg = viz.renderSVGElement(graph.dot);

            // graphviz stamps the drawing's own size on the element, which
            // would leave it a fixed pixel block inside the panel
            svg.removeAttribute("width");
            svg.removeAttribute("height");
            svg.setAttribute("width", "100%");
            svg.setAttribute("height", "100%");
            svg.setAttribute("preserveAspectRatio", "xMidYMid meet");

            if (!cancelled) {
                container.current?.replaceChildren(svg);
                setDrawn(true);
                setLoading(false);
            }
        } catch (err) {
            if (!cancelled) {
                setError(err instanceof Error ? err.message : "Unknown error");
                setLoading(false);
            }
        }

        return () => {
            cancelled = true;
        };
    }, [source]);

    useEffect(() => {
        // a slow draw must not overwrite a newer one that already finished
        const pending = draw();
        return () => {
            pending.then((cancel) => cancel());
        };
    }, [draw]);

    return (
        <Box
            sx={{
                position: "relative",
                flex: 1,
                minHeight: 0,
                width: "100%",
                overflow: "hidden",
                bgcolor: "background.default",
                // graphviz draws in black on white; the drawing takes the
                // page's colours instead, so it reads in either mode
                "& svg": { color: "text.primary" },
                "& svg [fill='white']": { fill: "transparent" },
                "& svg [stroke='black']": { stroke: "currentColor" },
                "& svg [fill='black']": { fill: "currentColor" },
                "& svg text": { fill: "currentColor" },
            }}
        >
            {loading &&
                (drawn ? (
                    // a redraw of a graph already on screen: out of the way,
                    // so the drawing stays readable while it is replaced
                    <CircularProgress
                        size={20}
                        sx={{
                            position: "absolute",
                            top: 12,
                            left: 12,
                            zIndex: 2,
                        }}
                    />
                ) : (
                    <Stack
                        sx={{ position: "absolute", inset: 0, zIndex: 2 }}
                        justifyContent={"center"}
                        alignItems={"center"}
                    >
                        <CircularProgress />
                    </Stack>
                ))}

            {error != null && (
                <Box
                    sx={{
                        position: "absolute",
                        // over a drawing it is a banner, so the graph below it
                        // can still be read and moved; with nothing drawn it
                        // is all there is to show
                        inset: drawn ? "8px 8px auto 8px" : 0,
                        zIndex: 2,
                        p: drawn ? 0 : 2,
                        pointerEvents: "none",
                    }}
                >
                    <Alert severity="error" sx={{ pointerEvents: "auto" }}>
                        Could not draw the graph: {error}
                        {drawn && ". The last drawing is still shown."}
                    </Alert>
                </Box>
            )}

            <TransformWrapper
                minScale={0.1}
                maxScale={8}
                limitToBounds={false}
                doubleClick={{ mode: "reset" }}
                wheel={{ step: 0.15 }}
            >
                {({ zoomIn, zoomOut, resetTransform }) => (
                    <>
                        <Stack
                            direction={"column"}
                            sx={{
                                position: "absolute",
                                top: 8,
                                right: 8,
                                zIndex: 3,
                                bgcolor: "background.paper",
                                borderRadius: 1,
                            }}
                        >
                            <IconButton
                                size="small"
                                onClick={() => setSource(code)}
                                title="Redraw now"
                            >
                                <RefreshRounded fontSize="small" />
                            </IconButton>
                            <IconButton size="small" onClick={() => zoomIn()}>
                                <AddRounded fontSize="small" />
                            </IconButton>
                            <IconButton size="small" onClick={() => zoomOut()}>
                                <RemoveRounded fontSize="small" />
                            </IconButton>
                            <IconButton
                                size="small"
                                onClick={() => resetTransform()}
                            >
                                <CenterFocusStrong fontSize="small" />
                            </IconButton>
                        </Stack>

                        <TransformComponent
                            wrapperStyle={{ width: "100%", height: "100%" }}
                            contentStyle={{ width: "100%", height: "100%" }}
                        >
                            <div
                                ref={container}
                                style={{ width: "100%", height: "100%" }}
                            />
                        </TransformComponent>
                    </>
                )}
            </TransformWrapper>
        </Box>
    );
};

export default GraphView;
