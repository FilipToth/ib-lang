import { auth } from "./firebase";
import firebase from 'firebase/compat/app';

const googleProvider = new firebase.auth.GoogleAuthProvider();

export const signInEmailPwd = async (email: string, pwd: string): Promise<firebase.auth.UserCredential | null> => {
    try {
        const credential = await auth.signInWithEmailAndPassword(email, pwd);
        return credential;
    } catch (err) {
        return null;
    }
}

export const signInWithGoogle = async (): Promise<firebase.auth.UserCredential | null> => {
    const credential = await auth.signInWithPopup(googleProvider);
    return credential;
};
