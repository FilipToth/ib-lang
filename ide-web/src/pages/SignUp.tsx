import { useNavigate } from "react-router-dom";
import { signUpEmailPwd } from "services/auth";
import Button from "@mui/material/Button";
import {
    Alert,
    Card,
    Link,
    Snackbar,
    Stack,
    TextField,
    Typography,
} from "@mui/material";
import React, { useEffect, useState } from "react";
import useAuthUser from "services/useAuthUser";
import AuthLoading from "components/AuthLoading";

const SignupPage = () => {
    const navigate = useNavigate();
    const { user, loading } = useAuthUser();

    useEffect(() => {
        if (user != null) {
            navigate("/", { replace: true });
        }
    }, [user, navigate]);

    const [email, setEmail] = useState("");
    const [pwd, setPwd] = useState("");
    const [confirmPwd, setConfirmPwd] = useState("");
    const [dialog, setDialog] = useState<String | null>(null);

    const showDialog = (msg: string) => {
        setDialog(msg);
        setTimeout(() => {
            setDialog(null);
        }, 3500);
    };

    const emailChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setEmail(e.target.value);
    };

    const pwdChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setPwd(e.target.value);
    };

    const confirmPwdChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setConfirmPwd(e.target.value);
    };

    const signUp = () => {
        if (pwd != confirmPwd) {
            showDialog("Passwords do not match.");
            return;
        }

        signUpEmailPwd(email, pwd).then((credential) => {
            if (credential == null) {
                showDialog("Sign up error. Please try again.");
                return;
            }

            navigate("/");
        });
    };

    if (loading || user != null) {
        return <AuthLoading />;
    }

    return (
        <>
            <Snackbar open={dialog != null}>
                <Alert variant="filled" severity="error">
                    {dialog}
                </Alert>
            </Snackbar>

            <Stack
                // at least the window's height, centring the card, but free
                // to grow and scroll when the window is shorter than it
                sx={{ minHeight: "100dvh", p: 2 }}
                justifyContent={"center"}
                alignItems={"center"}
            >
                <Card
                    // the full width on a phone, 400px on anything wider
                    style={{ width: "100%", maxWidth: "400px" }}
                    sx={{ bgcolor: "background.paper", p: 3 }}
                >
                    <Stack direction={"column"} spacing={2}>
                        <Typography variant="h4">Sign Up</Typography>

                        <TextField
                            label={"email"}
                            variant={"outlined"}
                            onChange={emailChange}
                        />
                        <TextField
                            label={"password"}
                            variant={"outlined"}
                            type={"password"}
                            onChange={pwdChange}
                        />
                        <TextField
                            label={"confirm password"}
                            variant={"outlined"}
                            type={"password"}
                            onChange={confirmPwdChange}
                        />

                        <Button variant="contained" onClick={signUp}>
                            Sign Up
                        </Button>
                        <Link align="center" href="/login">
                            Already have an account? Sign in!
                        </Link>
                    </Stack>
                </Card>
            </Stack>
        </>
    );
};

export default SignupPage;
