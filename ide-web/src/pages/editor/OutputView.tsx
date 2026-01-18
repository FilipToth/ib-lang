import { Box, Typography } from "@mui/material";
import { useLayoutEffect, useRef } from "react";

/// How close to the bottom, in pixels, still counts as at the bottom.
const bottomSlack = 16;

/// A program's output, as it printed it, following along as more arrives.
///
/// Following stops once the reader scrolls up to look at something, so new
/// output does not pull them away from it, and starts again when they scroll
/// back to the bottom or a new run clears the output.
const OutputView = ({ output }: { output: string }) => {
    const panel = useRef<HTMLDivElement | null>(null);
    const following = useRef(true);

    const onScroll = () => {
        const el = panel.current;
        if (el == null) return;

        following.current =
            el.scrollHeight - el.scrollTop - el.clientHeight <= bottomSlack;
    };

    // before paint, so new output never shows a frame scrolled short of it
    useLayoutEffect(() => {
        const el = panel.current;
        if (el == null) return;

        if (output == "") following.current = true;
        if (following.current) el.scrollTop = el.scrollHeight;
    }, [output]);

    return (
        <Box
            ref={panel}
            onScroll={onScroll}
            data-testid="output"
            sx={[
                {
                    flex: 1,
                    minHeight: 0,
                    // the text wraps to the panel rather than widening it: a
                    // zero width keeps long lines out of the layout, and the
                    // minimum stretches it back across
                    width: 0,
                    minWidth: "100%",
                    overflowY: "auto",
                    p: 1,
                    border: 1,
                    borderColor: "grey.300",
                    borderRadius: 1,
                    bgcolor: "grey.50",
                    fontFamily: "monospace",
                    fontSize: 14,
                    lineHeight: 1.5,
                    whiteSpace: "pre-wrap",
                    overflowWrap: "anywhere",
                },
                // set off from the page by a lighter surface on dark, as it is
                // by a greyer one on light
                (theme) =>
                    theme.applyStyles("dark", {
                        borderColor: "grey.800",
                        bgcolor: "background.paper",
                    }),
            ]}
        >
            {output == "" ? (
                <Typography
                    component="span"
                    sx={{ font: "inherit", color: "text.disabled" }}
                >
                    Output appears here when you run the program.
                </Typography>
            ) : (
                output
            )}
        </Box>
    );
};

export default OutputView;
