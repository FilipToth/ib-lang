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
import { auth } from "services/firebase";

const SignupPage = () => {
    const navigate = useNavigate();

    useEffect(() => {
        console.log(auth.currentUser);
        if (auth.currentUser != null) {
            navigate("/");
        }
    });

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

    return (
        <>
            <Snackbar open={dialog != null}>
                <Alert variant="filled" severity="error">
                    {dialog}
                </Alert>
            </Snackbar>

            <Stack
                sx={{ height: "100vh" }}
                justifyContent={"center"}
                alignItems={"center"}
            >
                <Card
                    style={{ maxWidth: "600px", width: "400px" }}
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
