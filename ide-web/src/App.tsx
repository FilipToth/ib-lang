import { BrowserRouter, Navigate, Outlet, Route, Routes } from 'react-router-dom';
import './App.css';
import Editor from './editor/Editor';
import React, { ReactNode } from 'react';
import LoginPage from 'pages/Login';

const PrivateRouteHandler = () => {
    // check if authed
    const loggedIn = false;
    return <>
        { !loggedIn
            ? <Navigate to={"/login"} />
            : <Outlet />
        }
    </>
};

const App = () => {
    return (
        <React.StrictMode>
            <BrowserRouter>
                <Routes>
                    <Route element={<PrivateRouteHandler/>}>
                        <Route path="/" element={<Editor />}></Route>
                    </Route>
                    <Route path='/login' element={<LoginPage />}></Route>
                </Routes>
            </BrowserRouter>
        </React.StrictMode>
    );
};

export default App;
