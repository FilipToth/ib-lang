import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
} from "@mui/material";

const DeleteFileDialog = ({
    isOpen,
    dialogOK,
    close,
}: {
    isOpen: boolean;
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
                    <Button onClick={close}>Cancel</Button>
                    <Button onClick={dialogOK}>Delete</Button>
                </DialogActions>
            </Dialog>
        </>
    );
};

export default DeleteFileDialog;
