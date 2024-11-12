import { useNavigate } from "react-router-dom";
import { signInEmailPwd, signInWithGoogle } from "services/auth";
import Button from '@mui/material/Button';
import { Alert, Card, Link, Snackbar, Stack, TextField, Typography } from "@mui/material";
import React, { useState } from "react";

const LoginPage = () => {
    const navigate = useNavigate();
    const [email, setEmail] = useState('')
    const [pwd, setPwd] = useState('')
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
            
            navigate('/')
        });
    };

    const signInGoogle = () => {
        signInWithGoogle().then((credential) => {
            if (credential == null)
                return;

            navigate('/')
        });
    };

    return (
        <>
            <Snackbar open={dialog != null}>
                <Alert variant="filled" severity="error">
                    {dialog}
                </Alert>
            </Snackbar>
            
            <Stack sx={{ height: "100vh" }} justifyContent={"center"} alignItems={"center"}>
                <Card style={{ maxWidth: "600px", width: "400px" }} sx={{ bgcolor: "background.paper", p: 3 }}>
                    <Stack direction={"column"} spacing={2}>
                        <Typography variant="h4">Sign In</Typography>
                        
                        <TextField label={"email"} variant={"outlined"} onChange={emailChange}/>
                        <TextField label={"password"} variant={"outlined"} type={"password"} onChange={pwdChange}/>
                        
                        <Button variant="contained" onClick={signIn}>Sign In</Button>
                        <Link align="center" href="/sign-up">
                            No account yet? Sign up!
                        </Link>
                        
                        <Typography align="center" variant="body1">or</Typography>
                        
                        <Button variant="outlined" onClick={signInGoogle}>Sign In With Google</Button>
                        <Button variant="outlined" onClick={signInGoogle}>Sign In With GitHub</Button>
                    </Stack>
                </Card>
            </Stack>
        </>
    )
};

export default LoginPage;