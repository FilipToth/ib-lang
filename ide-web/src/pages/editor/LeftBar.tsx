import { Box, IconButton, Stack, Typography } from "@mui/material";
import { IBFile } from "services/server";
import IbIcon from "./IbIcon";
import ClearIcon from "@mui/icons-material/Clear";

const LeftBar = ({
    files,
    click,
    del,
}: {
    files: IBFile[];
    click: (index: number) => void;
    del: (index: number) => void;
}) => {
    return (
        <Stack direction={"column"} height={"100vh"} width={"20vw"}>
            {files.map((file, index) => {
                return (
                    <BarEntry
                        file={file}
                        click={() => click(index)}
                        del={() => {
                            del(index);
                        }}
                    />
                );
            })}
        </Stack>
    );
};

const BarEntry = ({
    file,
    click,
    del,
}: {
    file: IBFile;
    click: () => void;
    del: () => void;
}) => {
    return (
        <Box
            sx={{
                display: "flex",
                flexDirection: "row",
                justifyContent: "space-between",
            }}
        >
            <Stack
                width={"100%"}
                sx={{
                    padding: 0.5,
                    paddingLeft: 2,
                    cursor: "pointer",
                }}
                onClick={click}
                direction={"row"}
                gap={1}
            >
                <IbIcon />
                <Typography>{file.filename}</Typography>
            </Stack>
            <IconButton onClick={del}>
                <ClearIcon />
            </IconButton>
        </Box>
    );
};

export default LeftBar;
