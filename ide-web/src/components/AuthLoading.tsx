import { CircularProgress, Stack } from "@mui/material";

/// Shown while firebase restores a stored session, so a reload does not flash
/// the sign-in page at someone who is already signed in.
const AuthLoading = () => {
    return (
        <Stack
            sx={{ height: "100vh" }}
            justifyContent={"center"}
            alignItems={"center"}
        >
            <CircularProgress />
        </Stack>
    );
};

export default AuthLoading;
