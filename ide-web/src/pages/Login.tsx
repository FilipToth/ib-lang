import { useNavigate } from "react-router-dom";
import {
    signInEmailPwd,
    signInWithGithub,
    signInWithGoogle,
} from "services/auth";
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

const LoginPage = () => {
    const navigate = useNavigate();
    const { user, loading } = useAuthUser();

    // someone who is already signed in has no business on this page
    useEffect(() => {
        if (user != null) {
            navigate("/", { replace: true });
        }
    }, [user, navigate]);

    const [email, setEmail] = useState("");
    const [pwd, setPwd] = useState("");
    const [dialog, setDialog] = useState<String | null>(null);

    const emailChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setEmail(e.target.value);
    };

    const pwdChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setPwd(e.target.value);
    };

    const signIn = () => {
        signInEmailPwd(email, pwd).then((credential) => {
            if (credential == null) {
                setDialog("Invalid email or password. Please try again.");
                setTimeout(() => {
                    setDialog(null);
                }, 3500);

                return;
            }

            navigate("/");
        });
    };

    const signInGoogle = () => {
        signInWithGoogle().then((credential) => {
            if (credential == null) return;

            navigate("/");
        });
    };

    const signInGithub = () => {
        signInWithGithub().then((credential) => {
            if (credential == null) return;

            navigate("/");
        });
    };

    // holding the form back until the session is known keeps it from flashing
    // in front of someone who is about to be redirected
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
                        <Typography variant="h4">Sign In</Typography>

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

                        <Button variant="contained" onClick={signIn}>
                            Sign In
                        </Button>
                        <Link align="center" href="/sign-up">
                            No account yet? Sign up!
                        </Link>

                        <Typography align="center" variant="body1">
                            or
                        </Typography>

                        <Button variant="outlined" onClick={signInGoogle}>
                            Sign In With Google
                        </Button>
                        <Button variant="outlined" onClick={signInGithub}>
                            Sign In With GitHub
                        </Button>
                    </Stack>
                </Card>
            </Stack>
        </>
    );
};

export default LoginPage;
