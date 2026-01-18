import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    Stack,
    TextField,
    Typography,
} from "@mui/material";
import { FormEvent, useEffect, useState } from "react";

/// Why `stem` cannot name a file, if it cannot. The server checks the same
/// rules; checking here as well answers before anything is sent.
const problemWith = (stem: string, takenNames: string[]): string | null => {
    if (stem == "") return "File names cannot be empty.";

    if (/[./\\]/.test(stem)) {
        return "File names cannot contain periods or slashes.";
    }

    if (stem.length + ".ib".length > 100) {
        return "File names can be at most 100 characters long.";
    }

    if (takenNames.includes(stem + ".ib")) {
        return `A file named ${stem}.ib already exists.`;
    }

    return null;
};

/// Asks for a file name, for a new file or a renamed one. The `.ib` extension
/// is added, not typed.
///
/// `dialogOK` gets the full name. The dialog stays open, its buttons held,
/// until that settles; if it throws, its message is shown and the name can be
/// changed and tried again.
const FileNameDialog = ({
    isOpen,
    title,
    confirmLabel,
    initialStem = "",
    takenNames,
    dialogOK,
    close,
}: {
    isOpen: boolean;
    title: string;
    confirmLabel: string;
    /// The name to start from, without its extension.
    initialStem?: string;
    /// Names already in use, with their extension.
    takenNames: string[];
    dialogOK: (filename: string) => Promise<void>;
    close: () => void;
}) => {
    const [stem, setStem] = useState(initialStem);
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);

    // each opening starts afresh
    useEffect(() => {
        if (!isOpen) return;

        setStem(initialStem);
        setError(null);
        setBusy(false);
    }, [isOpen, initialStem]);

    const submit = async (e: FormEvent) => {
        e.preventDefault();
        if (busy) return;

        const trimmed = stem.trim();
        const problem = problemWith(trimmed, takenNames);
        if (problem != null) {
            setError(problem);
            return;
        }

        setBusy(true);
        try {
            await dialogOK(trimmed + ".ib");
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setBusy(false);
        }
    };

    return (
        <Dialog
            open={isOpen}
            onClose={() => {
                if (!busy) close();
            }}
            fullWidth
            maxWidth="xs"
            // a form, so Enter submits it
            PaperProps={{ component: "form", onSubmit: submit }}
        >
            <DialogTitle>{title}</DialogTitle>
            <DialogContent>
                <Stack direction="row" alignItems="baseline" gap={0.5}>
                    <TextField
                        autoFocus
                        required
                        fullWidth
                        variant="standard"
                        value={stem}
                        disabled={busy}
                        error={error != null}
                        // stays until the name changes, rather than on a timer
                        helperText={error ?? " "}
                        onChange={(e) => {
                            setStem(e.target.value);
                            setError(null);
                        }}
                        // selected, so a new name can simply be typed over it
                        onFocus={(e) => e.target.select()}
                        slotProps={{ htmlInput: { "aria-label": "File name" } }}
                    />
                    <Typography>.ib</Typography>
                </Stack>
            </DialogContent>
            <DialogActions>
                <Button onClick={close} disabled={busy}>
                    Cancel
                </Button>
                <Button type="submit" variant="contained" disabled={busy}>
                    {confirmLabel}
                </Button>
            </DialogActions>
        </Dialog>
    );
};

export default FileNameDialog;
