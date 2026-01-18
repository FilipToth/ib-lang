import { Box, Button, Paper, Typography } from "@mui/material";

const EmptyWorkspace = ({ newFileClick }: { newFileClick: () => void }) => {
    return (
        <Box
            flex={1}
            minHeight={0}
            sx={{
                overflow: "auto",
                p: 2,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                bgcolor: "background.default",
            }}
        >
            <Paper
                variant="outlined"
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    borderRadius: "10px",
                    p: 4,
                    gap: 2,
                }}
            >
                <Typography>Get Started</Typography>
                <Button variant="contained" onClick={newFileClick}>
                    New File
                </Button>
            </Paper>
        </Box>
    );
};

export default EmptyWorkspace;
