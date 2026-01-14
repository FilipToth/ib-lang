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
import SignupPage from "pages/SignUp";
import useAuthUser from "services/useAuthUser";
import AuthLoading from "components/AuthLoading";

const PrivateRouteHandler = () => {
    const { user, loading } = useAuthUser();

    // a stored session is restored asynchronously, so redirecting before that
    // finishes is what signed people out whenever they reloaded
    if (loading) {
        return <AuthLoading />;
    }

    return user == null ? <Navigate to={"/login"} replace /> : <Outlet />;
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
