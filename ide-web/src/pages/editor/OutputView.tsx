import React from "react";
import { Box, Typography } from "@mui/material";
import { useLayoutEffect, useRef } from "react";

/// How close to the bottom, in pixels, still counts as at the bottom.
const bottomSlack = 16;

/// A piece of what a run has put out: what the program printed, what was
/// typed back at it, or a problem that stopped it.
export interface OutputEntry {
    kind: "output" | "input" | "error";
    text: string;
    /// Where in the code this piece came from, if anywhere. A piece that has
    /// one is clicked to go there.
    at?: CodeLocation;
}

/// A stretch of one file's source, as character offsets into the code that
/// was run.
export interface CodeLocation {
    fileId: string;
    start: number;
    end: number;
}

/// What each kind is drawn in. Input is set off from the program's own output
/// the way a terminal sets off what was typed.
const entryColour = {
    output: "inherit",
    input: "text.secondary",
    error: "error.main",
};

/// `entries` with `text` added to it. Text of the kind the last piece already
/// has joins it, rather than making a piece of its own.
export const appendEntry = (
    entries: OutputEntry[],
    kind: OutputEntry["kind"],
    text: string,
    at?: CodeLocation,
): OutputEntry[] => {
    const last = entries[entries.length - 1];

    // a piece that points somewhere is its own, so that clicking it goes to
    // that place and not to the one before it
    if (last?.kind != kind || last.at != null || at != null) {
        return [...entries, { kind, text, at }];
    }

    return [...entries.slice(0, -1), { kind, text: last.text + text }];
};

/// A program's output, as it printed it, with what was typed back at it and
/// anything that went wrong set apart, following along as more arrives.
///
/// Following stops once the reader scrolls up to look at something, so new
/// output does not pull them away from it, and starts again when they scroll
/// back to the bottom or a new run clears the output.
const OutputView = ({
    entries,
    goTo,
}: {
    entries: OutputEntry[];
    /// Called with where a clicked piece came from.
    goTo?: (at: CodeLocation) => void;
}) => {
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

        if (entries.length == 0) following.current = true;
        if (following.current) el.scrollTop = el.scrollHeight;
    }, [entries]);

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
            {entries.length == 0 ? (
                <Typography
                    component="span"
                    sx={{ font: "inherit", color: "text.disabled" }}
                >
                    Output appears here when you run the program.
                </Typography>
            ) : (
                entries.map((entry, index) => {
                    const at = entry.at;
                    const goes = at != null && goTo != null;

                    return (
                        <Box
                            component="span"
                            key={index}
                            data-kind={entry.kind}
                            role={goes ? "button" : undefined}
                            tabIndex={goes ? 0 : undefined}
                            title={goes ? "Go to this line" : undefined}
                            onClick={goes ? () => goTo(at) : undefined}
                            onKeyDown={
                                goes
                                    ? (e: React.KeyboardEvent) => {
                                          if (e.key != "Enter" && e.key != " ")
                                              return;

                                          e.preventDefault();
                                          goTo(at);
                                      }
                                    : undefined
                            }
                            sx={{
                                color: entryColour[entry.kind],
                                ...(goes && {
                                    cursor: "pointer",
                                    textDecoration: "underline",
                                    textDecorationStyle: "dotted",
                                    "&:hover": {
                                        textDecorationStyle: "solid",
                                    },
                                }),
                            }}
                        >
                            {entry.text}
                        </Box>
                    );
                })
            )}
        </Box>
    );
};

export default OutputView;
