import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import { ib } from "./ibSupport";
import { indentLess, indentMore, indentWithTab } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import React, { useEffect, useRef, useState } from "react";
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
import { AccountTree } from "@mui/icons-material";
import DeleteFileDialog from "pages/DeleteDialog";
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

/// How long typing has to pause before the open files are saved.
const saveDelay = 1000;

const EditorTabs = ({
    tabState,
    tabs,
    isDirty,
    changeTab,
    closeTab,
}: {
    tabState: number;
    tabs: EditorTab[];
    isDirty: (tab: EditorTab) => boolean;
    changeTab: (index: number) => void;
    closeTab: (index: number) => void;
}) => {
    return (
        <Tabs
            value={tabState}
            onChange={(_, index) => changeTab(index)}
            variant="scrollable"
            scrollButtons="auto"
            sx={{
                ...tabStyle,
            }}
        >
            {tabs.map((tab, index) => {
                const dirty = isDirty(tab);

                return (
                    <Tab
                        key={tabId(tab)}
                        value={index}
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

    /// The contents the server last confirmed storing, by file id. A file whose
    /// contents differ has unsaved changes.
    const [saved, setSaved] = useState<Record<string, string>>({});
    /// Files with a save request in flight, so a slow one is not sent again.
    const saving = useRef(new Set<string>());

    const isSaved = (file: IBFile) => saved[file.id] == file.contents;

    const isDirty = (tab: EditorTab) =>
        tab.kind == "file" && !isSaved(tab.file);

    const markSaved = (id: string, contents: string) => {
        setSaved((s) => ({ ...s, [id]: contents }));
    };

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
            const f = await getFiles();
            setFiles(f);
            setSaved(
                Object.fromEntries(f.map((file) => [file.id, file.contents]))
            );

            if (f.length == 0) return;

            const file = f[tabState];
            setCode(file.contents);

            currentFile = file;
        };

        loadFiles();
    }, []);

    /// Saves every file with unsaved changes once typing pauses. Files are
    /// saved whether or not their tab is still open, so closing one right
    /// after an edit does not lose it.
    useEffect(() => {
        const timeout = setTimeout(() => {
            for (const file of files) {
                if (isSaved(file) || saving.current.has(file.id)) continue;

                const contents = file.contents;
                saving.current.add(file.id);

                saveFile(file.id, contents)
                    .then(() => markSaved(file.id, contents))
                    // the dot stays, and the next edit tries again
                    .catch(() => setError(`Could not save ${file.filename}.`))
                    .finally(() => saving.current.delete(file.id));
            }
        }, saveDelay);

        return () => clearTimeout(timeout);
    }, [code, files, saved]);

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

    const ibSupport = ib();
    const keys = keymap.of([
        {
            key: "Tab",
            run: (e) => {
                if (!completionStatus(e.state)) return indentMore(e);

                return acceptCompletion(e);
            },
        },
    ]);

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

    const keyExtension = Prec.highest(keys);
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
                        <Stack direction="column" height={"100%"}>
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
                                    width="70vw"
                                    maxHeight="100%"
                                    theme={coolGlow}
                                    extensions={[
                                        ibSupport,
                                        keyExtension,
                                        indentUnit.of("    "),
                                        runtimeErrorHighlight(
                                            runtimeError,
                                            code.length
                                        ),
                                    ]}
                                    value={code}
                                    onChange={(
                                        value: string,
                                        _viewUpdate: ViewUpdate
                                    ) => {
                                        setCode(value);
                                        if (currentFile != null)
                                            currentFile.contents = value;

                                        // the highlight belongs to the source
                                        // that was run, so an edit retires it
                                        setRuntimeError(null);
                                    }}
                                    style={{
                                        flexGrow: 1,
                                        overflow: "scroll",
                                    }}
                                />
                            )}
                        </Stack>
                        {activeTab?.kind == "file" && (
                            <OutputBar
                                code={code}
                                fileId={activeTab.file.id}
                                onRuntimeError={setRuntimeError}
                            />
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
