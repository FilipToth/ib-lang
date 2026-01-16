import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import { ib } from "./ibSupport";
import { indentLess, indentMore, indentWithTab } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import React, { useEffect, useState } from "react";
import { TopBar } from "components/TopBar";
import {
    Box,
    Button,
    GlobalStyles,
    IconButton,
    Stack,
    SxProps,
    Tab,
    Tabs,
    Typography,
} from "@mui/material";
import { IBFile, createFile, deleteFile, getFiles } from "services/server";
import { Add, Clear } from "@mui/icons-material";
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

const EditorTabs = ({
    tabState,
    tabs,
    changeTab,
    closeTab,
}: {
    tabState: number;
    tabs: EditorTab[];
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
                                    {tabState == index && (
                                        <IconButton
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
                                    )}
                                </Box>
                            </span>
                        }
                        iconPosition="start"
                        sx={{
                            ...tabStyle,
                            textTransform: "none",
                            p: 1.5,
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

    const handleCreateFile = (filename: string) => {
        const uuid = uuidv4();
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

        setNewFileDialogOpen(false);

        createFile(uuid, filename);
    };

    const deleteFileClick = (index: number) => {
        setDelDialogIndex(index);
    };

    const deleteFileDialogOK = () => {
        if (delFileIndex == null) return;
        const file = files[delFileIndex];

        deleteFile(file.id);
        setDelDialogIndex(null);

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

            if (f.length == 0) return;

            const file = f[tabState];
            setCode(file.contents);

            currentFile = file;
        };

        loadFiles();
    }, []);

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
                    close={() => setDelDialogIndex(null)}
                    dialogOK={deleteFileDialogOK}
                />
            </Stack>
        </>
    );
};

export default Editor;
