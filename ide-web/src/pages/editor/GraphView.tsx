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

    // the graph is of the code as it was when the tab was opened or last
    // refreshed, so editing does not redraw under the reader
    const [source, setSource] = useState(code);

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
                flexGrow: 1,
                width: "70vw",
                overflow: "hidden",
                bgcolor: "#ffffff",
            }}
        >
            {loading && (
                <Stack
                    sx={{ position: "absolute", inset: 0, zIndex: 2 }}
                    justifyContent={"center"}
                    alignItems={"center"}
                >
                    <CircularProgress />
                </Stack>
            )}

            {error != null && (
                <Box sx={{ position: "absolute", inset: 0, zIndex: 2, p: 2 }}>
                    <Alert severity="error">
                        Could not draw the graph: {error}
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
                                title="Redraw from the current code"
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
