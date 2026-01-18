import OutputBar from "./OutputBar";
import {
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
    GlobalStyles,
    IconButton,
    Tooltip,
    Stack,
    Snackbar,
    Typography,
} from "@mui/material";
import {
    IBFile,
    createFile,
    deleteFile,
    failureReason,
    getFiles,
    renameFile,
    saveFile,
} from "services/server";
import { AccountTree, Add, VerticalSplit } from "@mui/icons-material";
import FileNameDialog from "./FileNameDialog";
import EmptyWorkspace from "./EmptyWorkspace";
import LeftBar from "./LeftBar";
import { RuntimeErrorRange } from "./runtimeError";
import { AutoSaver } from "./autosave";
import Splitter from "./Splitter";
import DeleteFileDialog from "pages/DeleteDialog";
import EditorPane from "./EditorPane";
import { TabDrag } from "./EditorTabs";
import {
    EditorTab,
    Pane,
    TabRef,
    activeTab,
    closeTab,
    eachTab,
    findTab,
    focusTab,
    graphTab,
    moveTab,
    openTab,
    singlePane,
} from "./panes";
import { DropTarget } from "./tabOrder";
import {
    loadStoredSession,
    restoreSession,
    sessionOf,
    storeSession,
} from "./session";
import { v4 as uuidv4 } from "uuid";

/// Widths the panels are dragged to are kept across reloads under these keys.
const outputWidthKey = "ib.outputWidth";
const splitWidthKey = "ib.splitWidth";

const minOutputWidth = 240;
const minSplitWidth = 280;
/// The narrowest the code can be squeezed by widening what is beside it.
const minCodeWidth = 320;

/// A width the panel keeps across reloads, starting at `fraction` of the
/// window.
const useStoredWidth = (key: string, min: number, fraction: number) => {
    const [width, setWidth] = useState(() => {
        try {
            const stored = Number(localStorage.getItem(key));
            if (stored >= min) return stored;
        } catch {
            // storage can be off, which leaves the default
        }

        return Math.max(min, Math.round(window.innerWidth * fraction));
    });

    const resize = (width: number) => {
        setWidth(width);

        try {
            localStorage.setItem(key, String(width));
        } catch {
            // storage can be off; the width then lasts only until a reload
        }
    };

    return [width, resize] as const;
};

/// Where a runtime error happened, and in which file: the panes each show the
/// highlight only if they hold that file.
interface FileRuntimeError {
    fileId: string;
    range: RuntimeErrorRange;
}

const Editor = () => {
    /// The editor is one pane, or two side by side. A tab is open in one of
    /// them, never both.
    const [panes, setPanes] = useState<Pane[]>(singlePane);
    /// The pane a new file or graph opens in.
    const [focused, setFocused] = useState(0);
    const [files, setFiles] = useState<IBFile[]>([]);
    const [runtimeError, setRuntimeError] = useState<FileRuntimeError | null>(
        null
    );
    const [newFileDialogOpen, setNewFileDialogOpen] = useState(false);
    /// The file the rename dialog is open for.
    const [renaming, setRenaming] = useState<IBFile | null>(null);
    const [delFileIndex, setDelDialogIndex] = useState<number | null>(null);
    const [deleting, setDeleting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [outputWidth, resizeOutput] = useStoredWidth(
        outputWidthKey,
        minOutputWidth,
        0.35
    );
    const [splitWidth, resizeSplit] = useStoredWidth(
        splitWidthKey,
        minSplitWidth,
        0.4
    );

    /// Bumped by every edit. Files are edited in place, so this is what tells
    /// the tabs, the graphs and the output panel that they have changed.
    const [, setEdits] = useState(0);

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

    /// An edit of `file` in one of the panes.
    const onEdit = useCallback(
        (file: IBFile, contents: string) => {
            file.contents = contents;
            saver.changed(file.id, contents);
            setEdits((n) => n + 1);

            // the highlight belongs to the source that was run
            setRuntimeError((e) => (e?.fileId == file.id ? null : e));
        },
        [saver]
    );

    const focusPane = (ref: TabRef) => {
        setPanes((ps) => focusTab(ps, ref));
        setFocused(ref.pane);
    };

    /// Moves a tab within its pane or to the other one, and follows it.
    const dragFrom = useRef<TabRef | null>(null);
    const drag: TabDrag = useMemo(
        () => ({
            begin: (from) => {
                dragFrom.current = from;
            },
            from: () => dragFrom.current,
            end: () => {
                dragFrom.current = null;
            },
            drop: (pane: number, target: DropTarget | null) => {
                const from = dragFrom.current;
                dragFrom.current = null;
                if (from == null) return;

                setPanes((ps) => {
                    const moved = moveTab(ps, from, pane, target);
                    setFocused(moved.focus.pane);

                    return moved.panes;
                });
            },
        }),
        []
    );

    const openFileOrChangeTab = (fileIndex: number) => {
        const file = files[fileIndex];
        const open = findTab(
            panes,
            (tab) => tab.kind == "file" && tab.file.id == file.id
        );

        // a file is open in one pane only; opening it again goes to it
        if (open != null) {
            focusPane(open);
            return;
        }

        setPanes(openTab(panes, focused, { kind: "file", file: file }));
    };

    /// The file whose graph a graph tab of the focused pane would draw: the
    /// open one, or the one open in the other pane when a graph is in front.
    const graphFile = (): IBFile | null => {
        const inFocus = activeTab(panes[focused]);
        if (inFocus?.kind == "file") return inFocus.file;

        const other = activeTab(panes[1 - focused]);
        return other?.kind == "file" ? other.file : null;
    };

    /// Opens the control flow graph of the open file beside it, splitting the
    /// editor, or returns to it when it is already open. Dragging the tab to
    /// the other pane afterwards leaves it a full-width tab of its own.
    const openGraph = () => {
        const file = graphFile();
        const tab = graphTab(file);

        const open = findTab(
            panes,
            (t) => t.kind == "graph" && t.id == tab.id
        );
        if (open != null) {
            focusPane(open);
            return;
        }

        // beside the code rather than over it: the other pane, made if the
        // editor is not split yet
        const beside = panes.length > 1 ? 1 - focused : focused + 1;

        setPanes(openTab(panes, beside, tab));
        setFocused(beside);
    };

    /// Moves the open tab to the other side, splitting the editor or, when
    /// the pane it leaves empties, ending the split.
    const moveToOtherSide = () => {
        const pane = panes[focused];
        if (activeTab(pane) == null) return;

        const other = panes.length > 1 ? 1 - focused : focused + 1;
        const moved = moveTab(
            panes,
            { pane: focused, index: pane.active },
            other,
            null
        );

        setPanes(moved.panes);
        setFocused(moved.focus.pane);
    };

    const changeTab = (pane: number, index: number) => {
        focusPane({ pane, index });
    };

    const closePaneTab = (pane: number, index: number) => {
        const next = closeTab(panes, { pane, index });

        setPanes(next);
        // the pane may have gone with its last tab
        setFocused((f) => Math.min(f, next.length - 1));
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
        } catch (err) {
            // shown in the dialog, which stays open to try another name
            throw new Error(failureReason(err));
        }

        const file: IBFile = {
            filename: filename,
            contents: "",
            id: uuid,
        };

        setFiles([...files, file]);
        setPanes(openTab(panes, focused, { kind: "file", file: file }));
        markSaved(uuid, "");
        saver.known(uuid, "");

        setNewFileDialogOpen(false);
    };

    /// Renames the file on the server, then everywhere it is shown. Tabs hold
    /// the file itself, so they follow; graph tabs carry a title of their own.
    const handleRenameFile = async (filename: string) => {
        const file = renaming;
        if (file == null) return;

        try {
            await renameFile(file.id, filename);
        } catch (err) {
            throw new Error(failureReason(err));
        }

        file.filename = filename;
        setFiles((fs) => [...fs]);
        setPanes((ps) =>
            ps.map((pane) => ({
                ...pane,
                tabs: pane.tabs.map((tab) =>
                    tab.kind == "graph" && tab.fileId == file.id
                        ? { ...tab, title: `${filename} flow` }
                        : tab
                ),
            }))
        );

        setRenaming(null);
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

        const open = findTab(
            panes,
            (tab) => tab.kind == "file" && tab.file.id == file.id
        );
        if (open != null) closePaneTab(open.pane, open.index);
    };

    /// Whether the tabs from last time have been reopened. Until they have,
    /// the panes are empty and must not be stored over them.
    const restored = useRef(false);

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

            const session = restoreSession(loadStoredSession(), f);
            setPanes(session.panes);
            setFocused(session.focused);
            restored.current = true;
        };

        loadFiles();
    }, [saver]);

    /// Keeps the open tabs for the next visit.
    useEffect(() => {
        if (!restored.current) return;

        storeSession(sessionOf(panes, focused));
    }, [panes, focused]);

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

    /// The source a graph draws, read from the file itself, so it follows
    /// edits made in the pane beside it.
    const graphSource = (fileId: string | null) => {
        if (fileId == null) return "";

        return files.find((f) => f.id == fileId)?.contents ?? "";
    };

    /// The file the output panel runs and reports on: the one in front of the
    /// focused pane, or the one the other pane holds while a graph is.
    const outputFile = graphFile();

    const paneActions = (pane: number) =>
        pane != focused ? null : (
            // icon buttons: a Button with only an icon keeps the width of a
            // text button
            <Box flexShrink={0} px={0.5}>
                <Tooltip title="Control flow graph">
                    <IconButton
                        size="small"
                        onClick={openGraph}
                        aria-label="Control flow graph"
                    >
                        <AccountTree fontSize="small" />
                    </IconButton>
                </Tooltip>
                <Tooltip
                    title={
                        panes.length > 1
                            ? "Move to the other side"
                            : "Split: move to the other side"
                    }
                >
                    <IconButton
                        size="small"
                        onClick={moveToOtherSide}
                        aria-label="Move the open tab to the other side"
                    >
                        <VerticalSplit fontSize="small" />
                    </IconButton>
                </Tooltip>
                <Tooltip title="New file">
                    <IconButton
                        size="small"
                        onClick={addFile}
                        aria-label="New file"
                    >
                        <Add fontSize="small" />
                    </IconButton>
                </Tooltip>
            </Box>
        );

    const renderPane = (pane: Pane, index: number) => (
        <EditorPane
            key={index}
            index={index}
            pane={pane}
            focused={index == focused}
            split={panes.length > 1}
            onFocus={() => setFocused(index)}
            isDirty={isDirty}
            drag={drag}
            changeTab={(tab) => changeTab(index, tab)}
            closeTab={(tab) => closePaneTab(index, tab)}
            onEdit={onEdit}
            runtimeError={
                runtimeError != null &&
                activeTabFileId(pane) == runtimeError.fileId
                    ? runtimeError.range
                    : null
            }
            graphSource={graphSource}
            actions={paneActions(index)}
        />
    );

    return (
        <>
            <GlobalStyles styles={{ body: { overflow: "hidden" } }} />
            {/* the page is exactly the window's height, and each pane scrolls
                on its own inside it. dvh rather than vh: on phones and
                tablets vh counts the space under the browser's toolbars too,
                which would push the bottom of the page out of sight */}
            <Stack height={"100dvh"} overflow={"hidden"}>
                <TopBar>
                    <Typography variant="h6" noWrap>
                        Code Editor
                    </Typography>
                </TopBar>
                {/* minHeight 0 throughout: a flex item will not shrink below
                    its content otherwise, so a long file or output would
                    stretch the page instead of scrolling */}
                <Stack direction="row" flex={1} minHeight={0}>
                    <LeftBar
                        files={files}
                        click={openFileOrChangeTab}
                        rename={(index) => setRenaming(files[index])}
                        del={deleteFileClick}
                    />
                    {eachTab(panes).length == 0 ? (
                        <EmptyWorkspace newFileClick={addFile} />
                    ) : (
                        renderPane(panes[0], 0)
                    )}
                    {panes.length > 1 && (
                        <>
                            <Splitter
                                width={splitWidth}
                                setWidth={resizeSplit}
                                minWidth={minSplitWidth}
                                minBefore={minCodeWidth}
                            />
                            <Box
                                sx={{
                                    display: "flex",
                                    flexShrink: 0,
                                    width: splitWidth,
                                    maxWidth: "60%",
                                }}
                            >
                                {renderPane(panes[1], 1)}
                            </Box>
                        </>
                    )}
                    {outputFile != null && (
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
                                    code={outputFile.contents}
                                    fileId={outputFile.id}
                                    filename={outputFile.filename}
                                    onRuntimeError={(range) =>
                                        setRuntimeError(
                                            range == null
                                                ? null
                                                : {
                                                      fileId: outputFile.id,
                                                      range: range,
                                                  }
                                        )
                                    }
                                />
                            </Box>
                        </>
                    )}
                </Stack>
                <FileNameDialog
                    isOpen={newFileDialogOpen}
                    title="New File"
                    confirmLabel="Create"
                    takenNames={files.map((f) => f.filename)}
                    close={() => setNewFileDialogOpen(false)}
                    dialogOK={handleCreateFile}
                />
                <FileNameDialog
                    isOpen={renaming != null}
                    title="Rename File"
                    confirmLabel="Rename"
                    initialStem={renaming?.filename.replace(/\.ib$/, "")}
                    // its own name is fine to keep
                    takenNames={files
                        .filter((f) => f.id != renaming?.id)
                        .map((f) => f.filename)}
                    close={() => setRenaming(null)}
                    dialogOK={handleRenameFile}
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

/// The file the pane has in front, if it is a file at all.
const activeTabFileId = (pane: Pane) => {
    const tab = activeTab(pane);
    return tab?.kind == "file" ? tab.file.id : null;
};

export default Editor;
