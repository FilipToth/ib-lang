import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
} from "@mui/material";

const NewFileDialog = ({
    isOpen,
    close,
}: {
    isOpen: boolean;
    ok: (file: string) => void;
    close: () => void;
}) => {
    return (
        <>
            <Dialog open={isOpen}>
                <DialogContent>
                    <DialogTitle>New File</DialogTitle>
                    <TextField
                        autoFocus
                        required
                        fullWidth
                        variant="standard"
                    />
                </DialogContent>
                <DialogActions>
                    <Button onClick={close}>Cancel</Button>
                    <Button>Done</Button>
                </DialogActions>
            </Dialog>
        </>
    );
};

export default NewFileDialog;
