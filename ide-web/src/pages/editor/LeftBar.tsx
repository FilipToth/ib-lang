import { Box, Stack, Typography } from "@mui/material";
import { IBFile } from "services/server";

const LeftBar = ({ files, click }: { files: IBFile[], click: (index: number) => void }) => {
    return (
        <Stack direction={"column"} height={"100vh"} width={"20vw"}>
            { files.map((file, index) => {
                return <BarEntry file={file} click={() => click(index)} />;
            })}
        </Stack>
    )
};

const BarEntry = ({ file, click }: { file: IBFile, click: () => void }) => {
    return (
        <Box
            width={"100%"}
            sx={{
                padding: 0.5,
                paddingLeft: 2,
                cursor: "pointer"
            }}
            onClick={click}
        >
            <Typography>{file.filename}</Typography>
        </Box>
    )
};

export default LeftBar;