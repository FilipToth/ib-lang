import { Box } from "@mui/material";
import React, { useRef } from "react";

/// How far an arrow key moves the splitter, in pixels.
const keyStep = 16;

/// A vertical bar between two panes that is dragged sideways to resize the
/// pane after it.
///
/// `width` is that pane's width. The splitter reports the width it is dragged
/// to through `setWidth`, kept between `minWidth` and whatever leaves the pane
/// before it `minBefore` wide.
const Splitter = ({
    width,
    setWidth,
    minWidth,
    minBefore,
}: {
    width: number;
    setWidth: (width: number) => void;
    minWidth: number;
    minBefore: number;
}) => {
    /// Where the drag started, and the room the pane before had to give.
    const drag = useRef<{
        startX: number;
        startWidth: number;
        maxWidth: number;
    } | null>(null);

    /// The widest the pane after can get, leaving the one before `minBefore`.
    const maxWidth = (splitter: HTMLElement) => {
        const before = splitter.previousElementSibling;
        const room = before?.getBoundingClientRect().width ?? 0;

        return Math.max(minWidth, width + room - minBefore);
    };

    const clamp = (w: number, max: number) =>
        Math.round(Math.min(Math.max(w, minWidth), max));

    const onPointerDown = (e: React.PointerEvent<HTMLElement>) => {
        // no text selection while dragging
        e.preventDefault();
        // keeps the moves coming when the pointer runs ahead of the bar, over
        // the editor or out of the window
        e.currentTarget.setPointerCapture(e.pointerId);

        drag.current = {
            startX: e.clientX,
            startWidth: width,
            maxWidth: maxWidth(e.currentTarget),
        };
    };

    const onPointerMove = (e: React.PointerEvent<HTMLElement>) => {
        const d = drag.current;
        if (d == null) return;

        // the pane is on the right, so dragging left widens it
        setWidth(clamp(d.startWidth - (e.clientX - d.startX), d.maxWidth));
    };

    const endDrag = () => {
        drag.current = null;
    };

    const onKeyDown = (e: React.KeyboardEvent<HTMLElement>) => {
        // left widens the pane after, as dragging left does
        const step =
            e.key == "ArrowLeft"
                ? keyStep
                : e.key == "ArrowRight"
                ? -keyStep
                : 0;
        if (step == 0) return;

        e.preventDefault();
        setWidth(clamp(width + step, maxWidth(e.currentTarget)));
    };

    return (
        <Box
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize the output panel"
            aria-valuenow={width}
            tabIndex={0}
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={endDrag}
            onPointerCancel={endDrag}
            onKeyDown={onKeyDown}
            sx={{
                flexShrink: 0,
                width: 8,
                cursor: "col-resize",
                touchAction: "none",
                position: "relative",
                outline: "none",
                // the visible line; the bar around it is the grab area
                "&::after": {
                    content: '""',
                    position: "absolute",
                    top: 0,
                    bottom: 0,
                    left: 3,
                    width: 2,
                    bgcolor: "divider",
                    transition: "background-color 0.15s",
                },
                "&:hover::after, &:active::after, &:focus-visible::after": {
                    bgcolor: "primary.main",
                },
            }}
        />
    );
};

export default Splitter;
