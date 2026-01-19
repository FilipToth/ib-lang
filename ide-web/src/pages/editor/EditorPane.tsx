import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { Box, Stack } from "@mui/material";
import { ReactNode, useCallback, useMemo, useRef } from "react";
import { indentLess, indentMore } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import { ib } from "./ibSupport";
import { IBFile } from "services/server";
import { useResolvedMode } from "theme";
import EditorTabs, { TabDrag } from "./EditorTabs";
import GraphView from "./GraphView";
import { EditorTab, Pane, activeTab } from "./panes";
import runtimeErrorHighlight, { RuntimeErrorRange } from "./runtimeError";
import { darkEditorTheme, lightEditorTheme } from "./editorTheme";
import { useFileBuffer } from "./fileBuffer";

/// The editor's extensions that never change. They are built once, because
/// CodeMirror reconfigures itself whenever it is handed new ones.
const baseExtensions = [
    ib(),
    Prec.highest(
        keymap.of([
            {
                key: "Tab",
                run: (e) => {
                    if (!completionStatus(e.state)) return indentMore(e);

                    return acceptCompletion(e);
                },
                shift: indentLess,
            },
        ]),
    ),
    indentUnit.of("    "),
];

/// One side of the editor: its tabs, and the file or graph the open one
/// shows.
const EditorPane = ({
    index,
    pane,
    focused,
    split,
    onFocus,
    isDirty,
    drag,
    changeTab,
    closeTab,
    onEdit,
    runtimeError,
    graphSource,
    actions,
}: {
    /// Which pane this is, for tabs dragged between them.
    index: number;
    pane: Pane;
    focused: boolean;
    /// Whether the editor is split, which is when it matters which pane is
    /// focused.
    split: boolean;
    onFocus: () => void;
    isDirty: (tab: EditorTab) => boolean;
    drag: TabDrag;
    changeTab: (index: number) => void;
    closeTab: (index: number) => void;
    onEdit: (file: IBFile, contents: string) => void;
    /// Where the last run of the open file failed, if it did.
    runtimeError: RuntimeErrorRange | null;
    graphSource: (fileId: string | null) => string;
    /// Buttons for the right of the tab row.
    actions?: ReactNode;
}) => {
    const mode = useResolvedMode();
    const tab = activeTab(pane);
    const file = tab?.kind == "file" ? tab.file : null;

    const [code, setCode] = useFileBuffer(file);

    /// For the change handler, which must keep its identity between renders:
    /// CodeMirror reconfigures itself whenever it is handed a new one.
    const editing = useRef(file);
    editing.current = file;

    const onChange = useCallback(
        (value: string, _viewUpdate: ViewUpdate) => {
            setCode(value);

            const file = editing.current;
            if (file != null) onEdit(file, value);
        },
        [onEdit],
    );

    // likewise rebuilt only when the highlight changes
    const extensions = useMemo(
        () => [...baseExtensions, runtimeErrorHighlight(runtimeError)],
        [runtimeError],
    );

    return (
        <Stack
            direction="column"
            flex={1}
            minWidth={0}
            minHeight={0}
            // any use of this pane makes it the one new tabs open in
            onMouseDownCapture={onFocus}
            onFocusCapture={onFocus}
        >
            <Box
                display="flex"
                flexDirection="row"
                alignItems="center"
                flexShrink={0}
                sx={{
                    borderBottom: 1,
                    // while split, the pane in use is marked, since that is
                    // where a new file or graph opens
                    borderColor:
                        split && focused ? "primary.main" : "transparent",
                }}
            >
                <EditorTabs
                    pane={index}
                    tabs={pane.tabs}
                    active={pane.active}
                    isDirty={isDirty}
                    changeTab={changeTab}
                    closeTab={closeTab}
                    drag={drag}
                />
                {actions}
            </Box>
            {tab?.kind == "graph" ? (
                // keyed so switching between graphs redraws rather than
                // reusing the previous one's state
                <GraphView key={tab.id} code={graphSource(tab.fileId)} />
            ) : (
                <CodeMirror
                    height="100%"
                    width="100%"
                    theme={mode == "dark" ? darkEditorTheme : lightEditorTheme}
                    extensions={extensions}
                    value={code}
                    onChange={onChange}
                    // the editor scrolls itself; its box only has to fill the
                    // space
                    style={{
                        flex: 1,
                        minHeight: 0,
                        overflow: "hidden",
                    }}
                />
            )}
        </Stack>
    );
};

export default EditorPane;
