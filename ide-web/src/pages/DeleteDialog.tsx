import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
} from "@mui/material";

const DeleteFileDialog = ({
    isOpen,
    busy,
    dialogOK,
    close,
}: {
    isOpen: boolean;
    /// While the delete is in flight, so it cannot be sent twice.
    busy: boolean;
    dialogOK: () => void;
    close: () => void;
}) => {
    return (
        <>
            <Dialog open={isOpen}>
                <DialogContent>
                    <DialogTitle>Are you sure?</DialogTitle>
                </DialogContent>
                <DialogActions>
                    <Button onClick={close} disabled={busy}>
                        Cancel
                    </Button>
                    <Button onClick={dialogOK} disabled={busy}>
                        Delete
                    </Button>
                </DialogActions>
            </Dialog>
        </>
    );
};

export default DeleteFileDialog;
