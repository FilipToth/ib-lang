import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import ib from "./ibSupport";
import { indentLess, indentMore, indentWithTab } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import React, { useEffect, useState } from "react";
import { TopBar } from "components/TopBar";
import {
    Box,
    Button,
    IconButton,
    Stack,
    SxProps,
    Tab,
    Tabs,
    Typography,
} from "@mui/material";
import { IBFile, createFile, getFiles } from "services/server";
import { Add, Clear } from "@mui/icons-material";
import NewFileDialog from "./NewFileDialog";
import EmptyWorkspace from "./EmptyWorkspace";
import LeftBar from "./LeftBar";
import IbIcon from "./IbIcon";
import DeleteFileDialog from "pages/DeleteDialog";
import { v4 as uuidv4 } from "uuid";

export let currentFile: IBFile | null = null;

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
    tabs: IBFile[];
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
            {tabs.map((file, index) => {
                return (
                    <Tab
                        key={file.id}
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
                                        <IbIcon />
                                        <Typography>{file.filename}</Typography>
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
    const [tabState, setTabState] = useState(0);
    const [tabs, setTabs] = useState<IBFile[]>([]);
    const [files, setFiles] = useState<IBFile[]>([]);
    const [newFileDialogOpen, setNewFileDialogOpen] = useState(false);
    const [delFileIndex, setDelDialogIndex] = useState<number | null>(null);

    const changeTab = (index: number) => {
        const currFile = tabs[tabState];
        currFile.contents = code;

        const file = tabs[index];
        currentFile = file;

        setCode(file.contents);
        setTabState(index);
    };

    const openFileOrChangeTab = (fileIndex: number) => {
        const file = files[fileIndex];
        const tabIndex = tabs.findIndex((item) => item.id == file.id);

        if (tabIndex != -1) {
            // change tabs
            currentFile = file;
            setCode(file.contents);
            setTabState(tabIndex);
            return;
        }

        setTabs([...tabs, file]);
        setTabState(tabs.length);
    };

    const closeTab = (index: number) => {
        const newIndex = index > 0 ? index - 1 : 0;
        currentFile = tabs[newIndex];
        setCode(currentFile.contents);
        setTabState(newIndex);

        setTabs((t) => {
            const x = t.filter((_, i) => i != index);
            console.log(x);
            return x;
        });
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
        setTabs([...tabs, file]);
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
        const index = delFileIndex;
        // TODO: Delete file in backend

        setDelDialogIndex(null);
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

    const keyExtension = Prec.highest(keys);

    return (
        <>
            <TopBar>
                <Typography variant="h6">Code Editor</Typography>
            </TopBar>
            <div>
                <Stack direction="row">
                    <LeftBar
                        files={files}
                        click={openFileOrChangeTab}
                        del={deleteFileClick}
                    />
                    <Stack direction="column">
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
                            <Button
                                onClick={addFile}
                                startIcon={<Add />}
                            ></Button>
                        </Box>
                        {tabs.length == 0 ? (
                            <EmptyWorkspace newFileClick={addFile} />
                        ) : (
                            <CodeMirror
                                height="100vh"
                                width="70vw"
                                maxHeight="100vh"
                                theme={coolGlow}
                                extensions={[
                                    ibSupport,
                                    keyExtension,
                                    indentUnit.of("    "),
                                ]}
                                value={code}
                                onChange={(
                                    value: string,
                                    _viewUpdate: ViewUpdate
                                ) => {
                                    setCode(value);
                                    if (currentFile != null)
                                        currentFile.contents = value;
                                }}
                            />
                        )}
                    </Stack>
                    <OutputBar code={code} />
                </Stack>
            </div>
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
        </>
    );
};

export default Editor;
