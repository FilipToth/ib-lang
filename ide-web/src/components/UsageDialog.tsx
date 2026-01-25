import {
    Alert,
    Button,
    CircularProgress,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    Divider,
    LinearProgress,
    Stack,
    Typography,
} from "@mui/material";
import { FunctionComponent, useEffect, useState } from "react";
import { IBAllowance, IBLimits, getLimits } from "services/server";
import { formatBytes, formatCount } from "./format";

/// How full an allowance is, capped at full: the server refuses past the cap,
/// but a count read a moment apart from its limit could still exceed it.
const fullness = (allowance: IBAllowance): number => {
    if (allowance.allowed <= 0) return 0;
    return Math.min(100, (allowance.used / allowance.allowed) * 100);
};

/// Warns before it refuses, so running out is not the first sign of a limit.
const barColour = (percent: number): "primary" | "warning" | "error" => {
    if (percent >= 90) return "error";
    if (percent >= 70) return "warning";
    return "primary";
};

const AllowanceBar = ({
    label,
    allowance,
    format,
}: {
    label: string;
    allowance: IBAllowance;
    format: (value: number) => string;
}) => {
    const percent = fullness(allowance);

    return (
        <Stack gap={0.5}>
            <Stack direction={"row"} justifyContent={"space-between"} gap={2}>
                <Typography variant="body2">{label}</Typography>
                <Typography variant="body2" color="text.secondary" noWrap>
                    {format(allowance.used)} of {format(allowance.allowed)}
                </Typography>
            </Stack>
            <LinearProgress
                variant="determinate"
                value={percent}
                color={barColour(percent)}
                aria-label={label}
            />
        </Stack>
    );
};

interface UsageDialogProps {
    open: boolean;
    onClose: () => void;
}

/// Shows what the account may use and what it has used.
///
/// Read when it opens rather than held: the run allowance refills on a timer
/// and the file counts change from another tab, so anything kept would be
/// wrong by the time it was looked at.
const UsageDialog: FunctionComponent<UsageDialogProps> = ({
    open,
    onClose,
}) => {
    const [limits, setLimits] = useState<IBLimits | null>(null);
    const [failed, setFailed] = useState(false);

    useEffect(() => {
        if (!open) return;

        let current = true;
        setLimits(null);
        setFailed(false);

        getLimits()
            .then((l) => {
                if (current) setLimits(l);
            })
            .catch(() => {
                if (current) setFailed(true);
            });

        // a dialog closed and reopened quickly must not be filled in by the
        // answer to the first request
        return () => {
            current = false;
        };
    }, [open]);

    return (
        <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
            <DialogTitle>Usage and limits</DialogTitle>
            <DialogContent>
                {failed && (
                    <Alert severity="error">
                        Could not load your usage. Try again in a moment.
                    </Alert>
                )}

                {!failed && limits == null && (
                    <Stack alignItems={"center"} py={3}>
                        <CircularProgress size={28} />
                    </Stack>
                )}

                {limits != null && (
                    <Stack gap={2.5} pt={1}>
                        <AllowanceBar
                            label="Files"
                            allowance={limits.files}
                            format={formatCount}
                        />
                        <AllowanceBar
                            label="Storage"
                            allowance={limits.bytes}
                            format={formatBytes}
                        />
                        <AllowanceBar
                            label={`Runs per ${limits.run_window_seconds} seconds`}
                            allowance={limits.runs}
                            format={formatCount}
                        />

                        <Divider />

                        {/* a ceiling each run starts from, not a balance that
                            is spent down, so there is nothing to fill */}
                        <Stack gap={0.5}>
                            <Typography variant="body2" fontWeight={600}>
                                Every run may use
                            </Typography>
                            <Typography variant="body2" color="text.secondary">
                                {formatCount(limits.steps_per_run)} steps, and{" "}
                                {formatCount(limits.elements_per_run)} items
                                stored in collections. A program that goes past
                                either is stopped.
                            </Typography>
                            <Typography variant="body2" color="text.secondary">
                                A single file may be up to{" "}
                                {formatBytes(limits.bytes_per_file)}.
                            </Typography>
                        </Stack>
                    </Stack>
                )}
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Close</Button>
            </DialogActions>
        </Dialog>
    );
};

export default UsageDialog;
