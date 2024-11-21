import CodeMirror, { Prec, ViewUpdate, keymap } from "@uiw/react-codemirror";
import { coolGlow } from "thememirror";
import ib from "./ibSupport";
import { indentLess, indentMore, indentWithTab } from "@codemirror/commands";
import { acceptCompletion, completionStatus } from "@codemirror/autocomplete";
import { indentUnit } from "@codemirror/language";
import OutputBar from "./OutputBar";
import React, { useState } from "react";
import { TopBar } from "components/TopBar";
import {
    Box,
    Icon,
    Stack,
    SvgIcon,
    SxProps,
    Tab,
    Tabs,
    Typography,
} from "@mui/material";

const Editor = () => {
    const [code, setCode] = useState("");
    const [tabState, setTabState] = useState(0);

    const changeTab = (_e: React.SyntheticEvent, val: number) => {
        setTabState(val);
    };

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

    const IbIcon = () => {
        return (
            <img
                src="assets/ib.png"
                style={{
                    width: "24px",
                    height: "24px",
                    backgroundColor: "transparent",
                }}
            />
        );
    };

    return (
        <>
            <TopBar>
                <Typography variant="h6">Code Editor</Typography>
            </TopBar>
            <div>
                <OutputBar code={code} />
                <Stack direction="column">
                    <Box>
                        <Tabs
                            value={tabState}
                            onChange={changeTab}
                            variant="scrollable"
                            scrollButtons="auto"
                            sx={{
                                ...tabStyle,
                            }}
                        >
                            <Tab
                                icon={<IbIcon />}
                                label="test.ib"
                                iconPosition="start"
                                sx={{
                                    ...tabStyle,
                                    gap: "8px",
                                }}
                            />
                        </Tabs>
                    </Box>
                    <CodeMirror
                        height="100vh"
                        width="90vw"
                        theme={coolGlow}
                        extensions={[
                            ibSupport,
                            keyExtension,
                            indentUnit.of("    "),
                        ]}
                        value={code}
                        onChange={(value: string, _viewUpdate: ViewUpdate) => {
                            setCode(value);
                        }}
                    />
                </Stack>
            </div>
        </>
    );
};

export default Editor;
