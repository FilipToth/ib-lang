import { Box, IconButton, Stack, Tooltip, Typography } from "@mui/material";
import { IBFile } from "services/server";
import IbIcon from "./IbIcon";
import ClearIcon from "@mui/icons-material/Clear";
import DriveFileRenameOutline from "@mui/icons-material/DriveFileRenameOutline";
import { surfaces } from "theme";

const LeftBar = ({
    files,
    click,
    rename,
    del,
}: {
    files: IBFile[];
    click: (index: number) => void;
    rename: (index: number) => void;
    del: (index: number) => void;
}) => {
    return (
        <Stack
            direction={"column"}
            sx={[
                {
                    // grows with the window, but stays usable on a small
                    // screen and does not take a quarter of a large one
                    width: "clamp(180px, 18vw, 280px)",
                    flexShrink: 0,
                    // the height is the row's; a long list scrolls in it
                    minHeight: 0,
                    overflowY: "auto",
                    py: 0.5,
                    bgcolor: surfaces.light.sidebar,
                    borderRight: 1,
                    borderColor: "divider",
                },
                (theme) =>
                    theme.applyStyles("dark", {
                        bgcolor: surfaces.dark.sidebar,
                    }),
            ]}
        >
            {files.map((file, index) => {
                return (
                    <BarEntry
                        key={file.id}
                        file={file}
                        click={() => click(index)}
                        rename={() => rename(index)}
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
    rename,
    del,
}: {
    file: IBFile;
    click: () => void;
    rename: () => void;
    del: () => void;
}) => {
    return (
        <Box
            sx={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                flexShrink: 0,
                pr: 0.5,
            }}
        >
            <Stack
                sx={{
                    // takes the row's width, leaving the delete button its
                    // own, and cuts a long name short rather than pushing it
                    // out
                    flex: 1,
                    minWidth: 0,
                    py: 0.5,
                    pl: 2,
                    cursor: "pointer",
                }}
                onClick={click}
                direction={"row"}
                alignItems={"center"}
                gap={1}
            >
                <IbIcon />
                <Typography noWrap title={file.filename}>
                    {file.filename}
                </Typography>
            </Stack>
            <Tooltip title="Rename">
                <IconButton
                    size="small"
                    onClick={rename}
                    aria-label={`Rename ${file.filename}`}
                >
                    <DriveFileRenameOutline fontSize="small" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Delete">
                <IconButton
                    size="small"
                    onClick={del}
                    aria-label={`Delete ${file.filename}`}
                >
                    <ClearIcon fontSize="small" />
                </IconButton>
            </Tooltip>
        </Box>
    );
};

export default LeftBar;
