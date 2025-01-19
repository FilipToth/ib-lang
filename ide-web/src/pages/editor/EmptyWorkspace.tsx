import { Box, Button, Stack, Typography } from "@mui/material";

const EmptyWorkspace = ({ newFileClick }: { newFileClick: () => void }) => {
    return (
        <Box
            width={"90vw"}
            height={"100vh"}
            sx={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                backgroundColor: "#060521"
            }}
        >
            <Stack sx={{ backgroundColor: "#FFFFFF", borderRadius: "10px", p: 4, gap: 2 }}>
                <Typography>Get Started</Typography>
                <Button variant="contained" onClick={newFileClick}>New File</Button>
            </Stack>
        </Box>
    );
};

export default EmptyWorkspace;