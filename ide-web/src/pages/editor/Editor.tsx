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
    Stack,
    SxProps,
    Tab,
    Tabs,
    Typography,
} from "@mui/material";
import { IBFile, getFiles } from "services/server";
import { Add } from "@mui/icons-material";
import NewFileDialog from "./NewFileDialog";
import EmptyWorkspace from "./EmptyWorkspace";
import LeftBar from "./LeftBar";
import IbIcon from "./IbIcon";
import DeleteFileDialog from "pages/DeleteDialog";

export let currentFile: string | null = null;

const Editor = () => {
    const [code, setCode] = useState("");
    const [tabState, setTabState] = useState(0);
    const [files, setFiles] = useState<IBFile[]>([]);
    const [newFileDialogOpen, setNewFileDialogOpen] = useState(false);
    const [deleteFileDialogIndex, setDelteFileDialogIndex] = useState<number | null>(null);

    const changeTab = (index: number) => {
        const currFile = files[tabState];
        currFile.contents = code;

        const file = files[index];
        currentFile = file.filename;

        setCode(file.contents);
        setTabState(index);
    };

    const addFile = () => {
        setNewFileDialogOpen(true);
    };

    const createFile = (filename: string) => {
        const file: IBFile = {
            filename: filename,
            contents: "",
        };

        setFiles([...files, file]);
        currentFile = filename;

        // files length isn't updated yet :D
        setTabState(files.length);
        setCode("");

        setNewFileDialogOpen(false);
    };

    const deleteFileClick = (index: number) => {
        setDelteFileDialogIndex(index);
    };

    const deleteFileDialogOK = () => {
        const index = deleteFileDialogIndex;
        // TODO: Delete file in backend

        setDelteFileDialogIndex(null);
    };

    useEffect(() => {
        const loadFiles = async () => {
            const f = await getFiles();
            setFiles(f);

            if (f.length == 0)
                return;

            const file = f[tabState];
            setCode(file.contents);

            currentFile = file.filename;
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

    const tabHeight = 30;
    const tabStyle: SxProps = {
        height: tabHeight,
        minHeight: tabHeight,
    };

    return (
        <>
            <TopBar>
                <Typography variant="h6">Code Editor</Typography>
            </TopBar>
            <div>
                <Stack direction="row">
                    <LeftBar
                        files={files}
                        click={changeTab}
                        del={deleteFileClick}
                    />
                    <Stack direction="column">
                        <Box
                            display="flex"
                            flexDirection="row"
                            justifyContent="space-between"
                        >
                            <Tabs
                                value={tabState}
                                onChange={(_, index) => changeTab(index)}
                                variant="scrollable"
                                scrollButtons="auto"
                                sx={{
                                    ...tabStyle,
                                }}
                            >
                                {files.map((file, index) => {
                                    return (
                                        <Tab
                                            value={index}
                                            icon={<IbIcon />}
                                            label={file.filename}
                                            iconPosition="start"
                                            sx={{
                                                ...tabStyle,
                                                gap: "8px",
                                                textTransform: "none",
                                            }}
                                        />
                                    );
                                })}
                            </Tabs>
                            <Button
                                onClick={addFile}
                                startIcon={<Add />}
                            ></Button>
                        </Box>
                        {
                            files.length == 0
                            ?   <EmptyWorkspace newFileClick={addFile} />
                            :   <CodeMirror
                                    height="100vh"
                                    width="70vw"
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
                                    }}
                                />
                        }
                    </Stack>
                    <OutputBar code={code} />
                </Stack>
            </div>
            <NewFileDialog
                isOpen={newFileDialogOpen}
                close={() => setNewFileDialogOpen(false)}
                dialogOK={createFile}
            />
            <DeleteFileDialog
                isOpen={deleteFileDialogIndex != null}
                close={() => setDelteFileDialogIndex(null)}
                dialogOK={deleteFileDialogOK}
            />
        </>
    );
};

export default Editor;
