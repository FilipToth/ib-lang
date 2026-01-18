import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import { ib } from "./ibSupport";
import { indentLess, indentMore } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import React, {
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import { TopBar } from "components/TopBar";
import {
    Alert,
    Box,
    Button,
    GlobalStyles,
    IconButton,
    Stack,
    Snackbar,
    SxProps,
    Tab,
    Tabs,
    Typography,
} from "@mui/material";
import {
    IBFile,
    createFile,
    deleteFile,
    getFiles,
    saveFile,
} from "services/server";
import { Add, Clear, FiberManualRecord } from "@mui/icons-material";
import NewFileDialog from "./NewFileDialog";
import EmptyWorkspace from "./EmptyWorkspace";
import LeftBar from "./LeftBar";
import IbIcon from "./IbIcon";
import runtimeErrorHighlight, { RuntimeErrorRange } from "./runtimeError";
import GraphView from "./GraphView";
import { AutoSaver } from "./autosave";
import Splitter from "./Splitter";
import { AccountTree } from "@mui/icons-material";
import DeleteFileDialog from "pages/DeleteDialog";
import { DropTarget, dropIndex, moveItem } from "./tabOrder";
import { v4 as uuidv4 } from "uuid";

export let currentFile: IBFile | null = null;

/// An open tab. Files are edited; graphs are drawn from the DOT the analyzer
/// produces, and are read-only.
type EditorTab =
    | { kind: "file"; file: IBFile }
    | { kind: "graph"; id: string; title: string; fileId: string | null };

const tabId = (tab: EditorTab) => (tab.kind == "file" ? tab.file.id : tab.id);

const tabTitle = (tab: EditorTab) =>
    tab.kind == "file" ? tab.file.filename : tab.title;

const tabHeight = 30;
const tabStyle: SxProps = {
    height: tabHeight,
    minHeight: tabHeight,
};

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
        ])
    ),
    indentUnit.of("    "),
];

/// The file icon for the drag picture, loaded up front. The browser takes the
/// picture as the drag starts, before a freshly made image could be drawn, so
/// a copy of the tab's own icon would come out blank.
const dragIcon = new Image(24, 24);
dragIcon.src = "assets/ib.png";

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
            background: "white",
            color: "rgba(0, 0, 0, 0.87)",
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
        dragImage.offsetHeight / 2
    );
};

/// The output panel's width is kept across reloads under this key.
const outputWidthKey = "ib.outputWidth";
const minOutputWidth = 240;
/// The narrowest the code can be squeezed by widening the output.
const minCodeWidth = 320;

const storedOutputWidth = () => {
    try {
        const stored = Number(localStorage.getItem(outputWidthKey));
        if (stored >= minOutputWidth) return stored;
    } catch {
        // storage can be off, which leaves the default
    }

    return Math.max(minOutputWidth, Math.round(window.innerWidth * 0.35));
};

const EditorTabs = ({
    tabState,
    tabs,
    isDirty,
    changeTab,
    closeTab,
    moveTab,
}: {
    tabState: number;
    tabs: EditorTab[];
    isDirty: (tab: EditorTab) => boolean;
    changeTab: (index: number) => void;
    closeTab: (index: number) => void;
    moveTab: (from: number, to: number) => void;
}) => {
    /// The tab being dragged. Kept here rather than in the drag data, which
    /// cannot be read until the drop, and so that drags from elsewhere (text,
    /// files) are ignored.
    const dragFrom = useRef<number | null>(null);
    const [dropTarget, setDropTarget] = useState<DropTarget | null>(null);

    const endDrag = () => {
        dragFrom.current = null;
        setDropTarget(null);
    };

    const drop = () => {
        const from = dragFrom.current;
        endDrag();

        if (from == null || dropTarget == null) return;

        moveTab(from, dropIndex(from, dropTarget));
    };

    return (
        <Tabs
            value={tabState}
            onChange={(_, index) => changeTab(index)}
            variant="scrollable"
            scrollButtons="auto"
            onDragLeave={(e) => {
                // dragleave also fires moving between a tab's children
                if (!e.currentTarget.contains(e.relatedTarget as Node | null)) {
                    setDropTarget(null);
                }
            }}
            sx={{
                ...tabStyle,
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
                            dragFrom.current = index;
                            e.dataTransfer.effectAllowed = "move";
                            // firefox only starts a drag that carries data;
                            // a type of its own keeps it from being dropped
                            // into the editor as text
                            e.dataTransfer.setData("application/x-ib-tab", "");
                            setTabDragImage(e);
                        }}
                        onDragOver={(e: React.DragEvent<HTMLElement>) => {
                            if (dragFrom.current == null) return;

                            // accepting the drop
                            e.preventDefault();
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
                                    onClick={(e) => changeTab(index)}
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
                                        <Typography>{tabTitle(tab)}</Typography>
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
                            p: 1.5,
                            // like vscode: the open tab shows its close
                            // button, an unsaved one a dot in its place, and
                            // hovering any tab turns either into the button
                            "& .close-button": {
                                display:
                                    tabState == index && !dirty
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

const Editor = () => {
    const [code, setCode] = useState("");
    const [runtimeError, setRuntimeError] = useState<RuntimeErrorRange | null>(
        null
    );
    const [tabState, setTabState] = useState(0);
    const [tabs, setTabs] = useState<EditorTab[]>([]);
    const [files, setFiles] = useState<IBFile[]>([]);
    const [newFileDialogOpen, setNewFileDialogOpen] = useState(false);
    const [delFileIndex, setDelDialogIndex] = useState<number | null>(null);
    const [deleting, setDeleting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [outputWidth, setOutputWidth] = useState(storedOutputWidth);

    const resizeOutput = (width: number) => {
        setOutputWidth(width);

        try {
            localStorage.setItem(outputWidthKey, String(width));
        } catch {
            // storage can be off; the width then lasts only until a reload
        }
    };

    /// The contents the server last confirmed storing, by file id. A file whose
    /// contents differ has unsaved changes.
    const [saved, setSaved] = useState<Record<string, string>>({});

    const isSaved = (file: IBFile) => saved[file.id] == file.contents;

    const isDirty = (tab: EditorTab) =>
        tab.kind == "file" && !isSaved(tab.file);

    const markSaved = (id: string, contents: string) => {
        setSaved((s) => ({ ...s, [id]: contents }));
    };

    /// For naming a file in an error that arrives after later renders.
    const filesRef = useRef(files);
    filesRef.current = files;

    /// Saves files as they are edited, retrying failed and hung saves until
    /// they go through. Made once, so edits and saves outlive any one render.
    const [saver] = useState(
        () =>
            new AutoSaver({
                save: saveFile,
                onSaved: markSaved,
                onError: (id, _error, failures) => {
                    // retries go on quietly; the dot shows it is still unsaved
                    if (failures != 1) return;

                    const file = filesRef.current.find((f) => f.id == id);
                    const name = file?.filename ?? "a file";
                    setError(`Could not save ${name}. Retrying…`);
                },
            })
    );

    /// The editor's buffer belongs to whichever file tab is open, so it has to
    /// be written back before the open tab changes.
    const saveActiveCode = () => {
        const active = tabs[tabState];
        if (active == null || active.kind != "file") return;

        active.file.contents = code;
    };

    /// Points the editor at `tab`, if it is a file. A graph tab leaves the
    /// buffer alone: it is opened alongside a file, not instead of one.
    const showTab = (tab: EditorTab | undefined) => {
        if (tab == null || tab.kind != "file") return;

        currentFile = tab.file;
        setCode(tab.file.contents);
    };

    const changeTab = (index: number) => {
        saveActiveCode();
        showTab(tabs[index]);
        setTabState(index);
    };

    const openFileOrChangeTab = (fileIndex: number) => {
        const file = files[fileIndex];
        const tabIndex = tabs.findIndex(
            (tab) => tab.kind == "file" && tab.file.id == file.id
        );

        if (tabIndex != -1) {
            changeTab(tabIndex);
            return;
        }

        saveActiveCode();
        setTabs([...tabs, { kind: "file", file: file }]);

        currentFile = file;
        setCode(file.contents);
        setTabState(tabs.length);
    };

    /// Opens the control flow graph of the file in the open tab, as a tab of
    /// its own, or returns to it when it is already open.
    const openGraph = () => {
        const active = tabs[tabState];
        const file = active?.kind == "file" ? active.file : null;
        const id = file == null ? "graph" : `graph:${file.id}`;

        const existing = tabs.findIndex(
            (tab) => tab.kind == "graph" && tab.id == id
        );

        if (existing != -1) {
            changeTab(existing);
            return;
        }

        saveActiveCode();

        const tab: EditorTab = {
            kind: "graph",
            id: id,
            title: file == null ? "Control Flow" : `${file.filename} flow`,
            fileId: file?.id ?? null,
        };

        setTabs([...tabs, tab]);
        setTabState(tabs.length);
    };

    /// Moves the tab at `from` to `to`, keeping the open tab open.
    const moveTab = (from: number, to: number) => {
        if (from == to) return;

        const active = tabs[tabState];
        const newTabs = moveItem(tabs, from, to);

        setTabs(newTabs);
        setTabState(newTabs.indexOf(active));
    };

    const closeTab = (index: number) => {
        const closed = tabs[index];
        const newTabs = tabs.filter((_, i) => i != index);

        // closing a tab before the open one shifts it left; closing the open
        // one falls back to its neighbour
        let newIndex = tabState;
        if (index < tabState) {
            newIndex = tabState - 1;
        } else if (index == tabState) {
            newIndex = index > 0 ? index - 1 : 0;
        }

        if (closed.kind == "file" && currentFile?.id == closed.file.id) {
            currentFile = null;
        }

        showTab(newTabs[newIndex]);

        setTabs(newTabs);
        setTabState(newIndex);
    };

    const addFile = () => {
        setNewFileDialogOpen(true);
    };

    /// Creates the file on the server first, and only then in the workspace, so
    /// a refused request leaves nothing behind. The dialog stays open until
    /// then, which keeps the tabs from changing under the request.
    const handleCreateFile = async (filename: string) => {
        const uuid = uuidv4();

        try {
            await createFile(uuid, filename);
        } catch {
            setNewFileDialogOpen(false);
            setError(`Could not create ${filename}.`);
            return;
        }

        const file: IBFile = {
            filename: filename,
            contents: "",
            id: uuid,
        };

        setFiles([...files, file]);
        setTabs([...tabs, { kind: "file", file: file }]);
        currentFile = file;

        // tabs length isn't updated yet :D
        setTabState(tabs.length);
        setCode("");
        markSaved(uuid, "");
        saver.known(uuid, "");

        setNewFileDialogOpen(false);
    };

    const deleteFileClick = (index: number) => {
        setDelDialogIndex(index);
    };

    /// Like creating, the file leaves the workspace only once the server has
    /// deleted it, and the dialog stays open meanwhile.
    const deleteFileDialogOK = async () => {
        if (delFileIndex == null) return;
        const file = files[delFileIndex];

        setDeleting(true);
        try {
            await deleteFile(file.id);
        } catch {
            setError(`Could not delete ${file.filename}.`);
            return;
        } finally {
            setDeleting(false);
            setDelDialogIndex(null);
        }

        saver.forget(file.id);
        setFiles((fs) => fs.filter((f) => f.id != file.id));

        // close tab
        const tabIndex = tabs.findIndex(
            (t) => t.kind == "file" && t.file.id == file.id
        );
        if (tabIndex != -1) {
            closeTab(tabIndex);
        }
    };

    useEffect(() => {
        const loadFiles = async () => {
            let f: IBFile[];
            try {
                f = await getFiles();
            } catch {
                setError("Could not load your files. Try reloading the page.");
                return;
            }

            setFiles(f);
            setSaved(
                Object.fromEntries(f.map((file) => [file.id, file.contents]))
            );
            f.forEach((file) => saver.known(file.id, file.contents));

            if (f.length == 0) return;

            const file = f[tabState];
            setCode(file.contents);

            currentFile = file;
        };

        loadFiles();
    }, []);

    /// Saves at once, rather than after the pause, when the page is hidden or
    /// left, on Ctrl/Cmd+S, and when the editor goes away (signing out).
    useEffect(() => {
        const onHidden = () => {
            if (document.visibilityState == "hidden") saver.flush();
        };

        const onKey = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() == "s") {
                // instead of the browser saving the page
                e.preventDefault();
                saver.flush();
            }
        };

        const flush = () => saver.flush();

        document.addEventListener("visibilitychange", onHidden);
        window.addEventListener("pagehide", flush);
        window.addEventListener("keydown", onKey);

        return () => {
            document.removeEventListener("visibilitychange", onHidden);
            window.removeEventListener("pagehide", flush);
            window.removeEventListener("keydown", onKey);
            saver.flush();
        };
    }, [saver]);

    const unsaved = files.some((file) => !isSaved(file));

    useEffect(() => {
        if (!unsaved) return;

        // the browser asks before leaving; the text it shows is its own
        const warn = (e: BeforeUnloadEvent) => {
            e.preventDefault();
            e.returnValue = "";
        };

        window.addEventListener("beforeunload", warn);
        return () => window.removeEventListener("beforeunload", warn);
    }, [unsaved]);

    // a new array would make CodeMirror reconfigure itself, so it is rebuilt
    // only when the highlight changes, not on every keystroke
    const extensions = useMemo(
        () => [...baseExtensions, runtimeErrorHighlight(runtimeError)],
        [runtimeError]
    );

    // CodeMirror reconfigures when this changes too, so it has to be stable
    const onChange = useCallback(
        (value: string, _viewUpdate: ViewUpdate) => {
            setCode(value);
            if (currentFile != null) {
                currentFile.contents = value;
                saver.changed(currentFile.id, value);
            }

            // the highlight belongs to the source that was run, so an edit
            // retires it
            setRuntimeError(null);
        },
        [saver]
    );

    /// The source a graph tab draws. `saveActiveCode` writes the buffer back
    /// whenever a tab changes, so the file's contents are current by the time
    /// its graph is on screen.
    const graphSource = (fileId: string | null) => {
        if (fileId == null) return "";

        const open = tabs.find(
            (tab) => tab.kind == "file" && tab.file.id == fileId
        );

        if (open?.kind == "file") return open.file.contents;

        // the file's tab may have been closed while its graph stayed open
        return files.find((f) => f.id == fileId)?.contents ?? "";
    };

    const activeTab = tabs[tabState];

    return (
        <>
            <GlobalStyles styles={{ body: { overflow: "hidden" } }} />
            <Stack height={"100vh"} maxHeight={"100vh"} overflow={"hidden"}>
                <TopBar>
                    <Typography variant="h6">Code Editor</Typography>
                </TopBar>
                <Stack flex={1} overflow={"hidden"}>
                    <Stack direction="row" height={"100%"}>
                        <LeftBar
                            files={files}
                            click={openFileOrChangeTab}
                            del={deleteFileClick}
                        />
                        <Stack
                            direction="column"
                            height={"100%"}
                            flex={1}
                            minWidth={0}
                        >
                            <Box
                                display="flex"
                                flexDirection="row"
                                justifyContent="space-between"
                            >
                                <EditorTabs
                                    tabState={tabState}
                                    tabs={tabs}
                                    isDirty={isDirty}
                                    changeTab={changeTab}
                                    closeTab={closeTab}
                                    moveTab={moveTab}
                                />
                                <Box>
                                    <Button
                                        onClick={openGraph}
                                        startIcon={<AccountTree />}
                                        title="Control flow graph"
                                    ></Button>
                                    <Button
                                        onClick={addFile}
                                        startIcon={<Add />}
                                    ></Button>
                                </Box>
                            </Box>
                            {tabs.length == 0 ? (
                                <EmptyWorkspace newFileClick={addFile} />
                            ) : activeTab?.kind == "graph" ? (
                                // keyed so switching between graphs redraws
                                // rather than reusing the previous one's state
                                <GraphView
                                    key={activeTab.id}
                                    code={graphSource(activeTab.fileId)}
                                />
                            ) : (
                                <CodeMirror
                                    height="100%"
                                    width="100%"
                                    maxHeight="100%"
                                    theme={coolGlow}
                                    extensions={extensions}
                                    value={code}
                                    onChange={onChange}
                                    style={{
                                        flexGrow: 1,
                                        overflow: "scroll",
                                    }}
                                />
                            )}
                        </Stack>
                        {activeTab?.kind == "file" && (
                            <>
                                <Splitter
                                    width={outputWidth}
                                    setWidth={resizeOutput}
                                    minWidth={minOutputWidth}
                                    minBefore={minCodeWidth}
                                />
                                <Box
                                    sx={{
                                        display: "flex",
                                        flexShrink: 0,
                                        width: outputWidth,
                                        // a narrowed window takes room from
                                        // the output before hiding the code
                                        maxWidth: "60%",
                                    }}
                                >
                                    <OutputBar
                                        code={code}
                                        fileId={activeTab.file.id}
                                        filename={activeTab.file.filename}
                                        onRuntimeError={setRuntimeError}
                                    />
                                </Box>
                            </>
                        )}
                    </Stack>
                </Stack>
                <NewFileDialog
                    isOpen={newFileDialogOpen}
                    close={() => setNewFileDialogOpen(false)}
                    dialogOK={handleCreateFile}
                />
                <DeleteFileDialog
                    isOpen={delFileIndex != null}
                    busy={deleting}
                    close={() => setDelDialogIndex(null)}
                    dialogOK={deleteFileDialogOK}
                />
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
            </Stack>
        </>
    );
};

export default Editor;
