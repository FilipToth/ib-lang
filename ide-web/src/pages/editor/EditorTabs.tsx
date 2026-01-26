import React, { useState } from "react";
import { Box, IconButton, SxProps, Tab, Tabs, Typography } from "@mui/material";
import { AccountTree, Clear, FiberManualRecord } from "@mui/icons-material";
import IbIcon, { ibIconSrc } from "./IbIcon";
import { EditorTab, TabRef, tabId, tabTitle } from "./panes";
import { DropTarget } from "./tabOrder";

export const tabHeight = 36;
const tabStyle: SxProps = {
    height: tabHeight,
    minHeight: tabHeight,
};

/// A tab being dragged, which may have started in the other pane. It is held
/// by the editor rather than in the drag's own data, which cannot be read
/// until the drop, and so that drags from elsewhere (text, files) are
/// ignored.
export interface TabDrag {
    begin: (from: TabRef) => void;
    from: () => TabRef | null;
    /// Dropped on pane `pane`, beside `target`, or past its tabs when that is
    /// null.
    drop: (pane: number, target: DropTarget | null) => void;
    end: () => void;
}

/// The file icon for the drag picture, loaded up front. The browser takes the
/// picture as the drag starts, before a freshly made image could be drawn, so
/// a copy of the tab's own icon would come out blank.
const dragIcon = new Image(24, 24);
dragIcon.src = ibIconSrc;

/// Holds the drag picture. It stays in the page between drags, so the icon
/// in it never has to be drawn anew.
let dragImage: HTMLDivElement | null = null;

/// Makes the picture that follows the cursor while a tab is dragged just its
/// icon and name. Left to itself, the browser snapshots the tab's whole area,
/// which picks up the code beneath it.
const setTabDragImage = (e: React.DragEvent<HTMLElement>) => {
    const label = e.currentTarget.querySelector(".tab-label");
    if (label == null) return;

    if (dragImage == null) {
        dragImage = document.createElement("div");
        Object.assign(dragImage.style, {
            position: "fixed",
            // it has to be rendered to be pictured, but not where it can be
            // seen
            top: "-1000px",
            left: "-1000px",
            display: "flex",
            padding: "4px 8px",
            borderRadius: "4px",
            // the page's colours, whichever mode it is in
            background: "var(--mui-palette-background-paper)",
            color: "var(--mui-palette-text-primary)",
            border: "1px solid var(--mui-palette-divider)",
        });
        document.body.appendChild(dragImage);
    }

    const copy = label.cloneNode(true) as HTMLElement;
    copy.querySelector("img")?.replaceWith(dragIcon);
    dragImage.replaceChildren(copy);

    // held where it was grabbed, relative to the tab
    const rect = e.currentTarget.getBoundingClientRect();
    e.dataTransfer.setDragImage(
        dragImage,
        Math.min(e.clientX - rect.left, dragImage.offsetWidth),
        dragImage.offsetHeight / 2,
    );
};

/// One pane's row of tabs. Tabs are reordered by dragging, and dragging one
/// onto the other pane's row moves it there.
const EditorTabs = ({
    pane,
    tabs,
    active,
    isDirty,
    changeTab,
    closeTab,
    drag,
}: {
    /// Which pane these tabs belong to.
    pane: number;
    tabs: EditorTab[];
    active: number;
    isDirty: (tab: EditorTab) => boolean;
    changeTab: (index: number) => void;
    closeTab: (index: number) => void;
    drag: TabDrag;
}) => {
    const [dropTarget, setDropTarget] = useState<DropTarget | null>(null);

    const endDrag = () => {
        drag.end();
        setDropTarget(null);
    };

    const drop = () => {
        drag.drop(pane, dropTarget);
        setDropTarget(null);
    };

    return (
        <Tabs
            value={active < tabs.length ? active : false}
            onChange={(_, index) => changeTab(index)}
            variant="scrollable"
            scrollButtons="auto"
            // past the tabs: the dragged one goes to the end of this pane
            onDragOver={(e) => {
                if (drag.from() == null) return;

                e.preventDefault();
                e.dataTransfer.dropEffect = "move";
                setDropTarget(null);
            }}
            onDrop={(e) => {
                e.preventDefault();
                drop();
            }}
            onDragLeave={(e) => {
                // dragleave also fires moving between a tab's children
                if (!e.currentTarget.contains(e.relatedTarget as Node | null)) {
                    setDropTarget(null);
                }
            }}
            sx={{
                ...tabStyle,
                // takes the row's spare width, and scrolls when the tabs
                // need more
                flex: 1,
                minWidth: 0,
            }}
        >
            {tabs.map((tab, index) => {
                const dirty = isDirty(tab);

                const marker =
                    dropTarget?.index == index ? dropTarget.side : null;

                return (
                    <Tab
                        key={tabId(tab)}
                        value={index}
                        // firefox does not drag buttons, which a tab
                        // renders as by default
                        component="div"
                        draggable
                        onDragStart={(e: React.DragEvent<HTMLElement>) => {
                            drag.begin({ pane, index });
                            e.dataTransfer.effectAllowed = "move";
                            // firefox only starts a drag that carries data;
                            // a type of its own keeps it from being dropped
                            // into the editor as text
                            e.dataTransfer.setData("application/x-ib-tab", "");
                            setTabDragImage(e);
                        }}
                        onDragOver={(e: React.DragEvent<HTMLElement>) => {
                            if (drag.from() == null) return;

                            // accepting the drop, and keeping the row's own
                            // handler from taking it as a drop past the tabs
                            e.preventDefault();
                            e.stopPropagation();
                            e.dataTransfer.dropEffect = "move";

                            const { left, width } =
                                e.currentTarget.getBoundingClientRect();
                            const side =
                                e.clientX < left + width / 2
                                    ? "before"
                                    : "after";

                            if (
                                dropTarget?.index != index ||
                                dropTarget.side != side
                            ) {
                                setDropTarget({ index, side });
                            }
                        }}
                        onDrop={(e: React.DragEvent) => {
                            e.preventDefault();
                            e.stopPropagation();
                            drop();
                        }}
                        onDragEnd={endDrag}
                        label={
                            <span>
                                <Box
                                    sx={{
                                        display: "flex",
                                        flexDirection: "row",
                                        justifyContent: "space-between",
                                        alignItems: "center",
                                        gap: 1,
                                    }}
                                    onClick={() => changeTab(index)}
                                >
                                    <Box
                                        className="tab-label"
                                        sx={{
                                            display: "flex",
                                            flexDirection: "row",
                                            alignItems: "center",
                                            gap: 1,
                                        }}
                                    >
                                        {tab.kind == "file" ? (
                                            <IbIcon />
                                        ) : (
                                            <AccountTree fontSize="small" />
                                        )}
                                        <Typography
                                            noWrap
                                            sx={{ maxWidth: 240 }}
                                        >
                                            {tabTitle(tab)}
                                        </Typography>
                                    </Box>
                                    {/* the slot keeps its width whatever
                                        it shows, so hovering does not
                                        shift the tabs */}
                                    <Box
                                        sx={{
                                            display: "flex",
                                            alignItems: "center",
                                            justifyContent: "center",
                                            width: 24,
                                            height: 24,
                                        }}
                                    >
                                        {dirty && (
                                            <FiberManualRecord
                                                className="dirty-dot"
                                                titleAccess="Unsaved changes"
                                                sx={{ fontSize: 12 }}
                                            />
                                        )}
                                        <IconButton
                                            className="close-button"
                                            onClick={(e) => {
                                                // prevent mui tab switches
                                                e.stopPropagation();
                                                closeTab(index);
                                            }}
                                            sx={{
                                                p: 0,
                                            }}
                                        >
                                            <Clear />
                                        </IconButton>
                                    </Box>
                                </Box>
                            </span>
                        }
                        iconPosition="start"
                        sx={{
                            ...tabStyle,
                            textTransform: "none",
                            // no vertical padding: the height above sets
                            // it, and padding would squeeze the label
                            py: 0,
                            px: 1.5,
                            // like vscode: the open tab shows its close
                            // button, an unsaved one a dot in its place, and
                            // hovering any tab turns either into the button
                            "& .close-button": {
                                display:
                                    active == index && !dirty
                                        ? "inline-flex"
                                        : "none",
                            },
                            "&:hover .close-button": {
                                display: "inline-flex",
                            },
                            "&:hover .dirty-dot": {
                                display: "none",
                            },
                            // a line on the side the dragged tab would land
                            boxShadow:
                                marker == "before"
                                    ? "inset 2px 0 0 currentColor"
                                    : marker == "after"
                                      ? "inset -2px 0 0 currentColor"
                                      : "none",
                        }}
                    />
                );
            })}
        </Tabs>
    );
};

export default EditorTabs;
