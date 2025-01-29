import cors from "cors";
import express from "express";
import admin from "firebase-admin";

const app = express();
const port = 8081;

app.use(
    cors({
        origin: true,
        credentials: true,
    })
);

const firebaseConfig = {
    apiKey: "AIzaSyDxUaBg4dtk4rkrnJOGwJm-vtusX6nPMWI",
    authDomain: "ib-web-ide.firebaseapp.com",
    projectId: "ib-web-ide",
    storageBucket: "ib-web-ide.firebasestorage.app",
    messagingSenderId: "63837561484",
    appId: "1:63837561484:web:39fc5f6f3899dd482591a9",
    measurementId: "G-0RFF0MMD1D",
};

admin.initializeApp(firebaseConfig);
const firebase = admin.auth();

app.get("/auth", async (req, res) => {
    const auth = req.headers.authorization;
    if (!auth || !auth.startsWith("Bearer ")) {
        res.sendStatus(401);
        return;
    }

    const jwt = auth.split(" ")[1];
    try {
        const decoded = await firebase.verifyIdToken(jwt);
        const uid = decoded.uid;
        res.send({ uid: uid }).status(200);
    } catch (error) {
        res.sendStatus(401);
    }
});

app.get("/ping", async (req, res) => {
    res.send({ msg: "pong" });
});

app.listen(port, () => {
    console.log(`listening on port ${port}...`);
});
