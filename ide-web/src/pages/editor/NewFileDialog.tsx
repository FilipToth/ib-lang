import {
    Box,
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    Stack,
    TextField,
    Typography,
} from "@mui/material";
import { useState } from "react";

const NewFileDialog = ({
    isOpen,
    dialogOK,
    close,
}: {
    isOpen: boolean;
    dialogOK: (file: string) => void;
    close: () => void;
}) => {
    const [filename, setFilename] = useState("");
    const [error, setError] = useState<string | null>(null);

    const createError = (err: string) => {
        setError(err);
        setTimeout(() => {
            setError(null);
        }, 2500);
    };

    const doneClick = () => {
        const filtered = filename.trim();
        if (filtered == "") {
            createError("File name cannot be empty");
            return;
        }

        if (filtered.includes(".")) {
            createError("File name cannot contain periods");
            return;
        }

        dialogOK(filtered + ".ib");

        setError("");
        setFilename("");
    };

    return (
        <>
            <Dialog open={isOpen}>
                <DialogContent>
                    <DialogTitle>New File</DialogTitle>
                    <Stack direction="row">
                        <TextField
                            autoFocus
                            required
                            fullWidth
                            value={filename}
                            onChange={(e) => setFilename(e.target.value)}
                            variant="standard"
                        />
                        <Typography>.ib</Typography>
                    </Stack>
                    {error != null && (
                        // spacer
                        <>
                            <Box sx={{ height: "12px" }} />
                            <Typography color="red">{error}</Typography>
                        </>
                    )}
                </DialogContent>
                <DialogActions>
                    <Button onClick={close}>Cancel</Button>
                    <Button onClick={doneClick}>Done</Button>
                </DialogActions>
            </Dialog>
        </>
    );
};

export default NewFileDialog;
