import {
    BrowserRouter,
    Navigate,
    Outlet,
    Route,
    Routes,
} from "react-router-dom";
import "./App.css";
import Editor from "./pages/editor/Editor";
import React, { ReactNode } from "react";
import LoginPage from "pages/Login";
import { auth } from "services/firebase";
import SignupPage from "pages/SignUp";

const PrivateRouteHandler = () => {
    // check if authed
    console.log(auth.currentUser);
    return (
        <>
            {auth.currentUser == null ? <Navigate to={"/login"} /> : <Outlet />}
        </>
    );
};

const App = () => {
    return (
        <React.StrictMode>
            <BrowserRouter>
                <Routes>
                    <Route element={<PrivateRouteHandler />}>
                        <Route path="/" element={<Editor />}></Route>
                    </Route>
                    <Route path="/login" element={<LoginPage />}></Route>
                    <Route path="/sign-up" element={<SignupPage />}></Route>
                </Routes>
            </BrowserRouter>
        </React.StrictMode>
    );
};

export default App;
