import firebase from "firebase/compat/app";
import "firebase/compat/auth";
import {
    browserLocalPersistence,
    getAuth,
    setPersistence,
} from "firebase/auth";

const firebaseConfig = {
    apiKey: "AIzaSyDxUaBg4dtk4rkrnJOGwJm-vtusX6nPMWI",
    authDomain: "ib-web-ide.firebaseapp.com",
    projectId: "ib-web-ide",
    storageBucket: "ib-web-ide.firebasestorage.app",
    messagingSenderId: "63837561484",
    appId: "1:63837561484:web:39fc5f6f3899dd482591a9",
    measurementId: "G-0RFF0MMD1D",
};

const app = firebase.initializeApp(firebaseConfig);

const authProvider = getAuth(app);
setPersistence(authProvider, browserLocalPersistence);

export const auth = authProvider;
