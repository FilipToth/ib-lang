import { auth } from "./firebase";
import firebase from "firebase/compat/app";
import {
    signInWithEmailAndPassword,
    createUserWithEmailAndPassword,
    signInWithPopup,
    UserCredential,
} from "firebase/auth";

const googleProvider = new firebase.auth.GoogleAuthProvider();
const githubProvider = new firebase.auth.GithubAuthProvider();

export const signInEmailPwd = async (
    email: string,
    pwd: string
): Promise<UserCredential | null> => {
    try {
        const credential = await signInWithEmailAndPassword(auth, email, pwd);
        return credential;
    } catch (err) {
        return null;
    }
};

export const signUpEmailPwd = async (
    email: string,
    pwd: string
): Promise<UserCredential | null> => {
    try {
        const credential = await createUserWithEmailAndPassword(
            auth,
            email,
            pwd
        );

        return credential;
    } catch (err) {
        return null;
    }
};

export const signInWithGoogle = async (): Promise<UserCredential | null> => {
    const credential = await signInWithPopup(auth, googleProvider);
    return credential;
};

export const signInWithGithub = async (): Promise<UserCredential | null> => {
    const credential = await signInWithPopup(auth, githubProvider);
    return credential;
};
