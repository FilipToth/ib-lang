import { BrowserRouter, Navigate, Outlet, Route, Routes } from 'react-router-dom';
import './App.css';
import Editor from './editor/Editor';
import React, { ReactNode } from 'react';
import LoginPage from 'pages/Login';
import { auth } from 'services/firebase';

const PrivateRouteHandler = () => {
    // check if authed
    console.log(auth.currentUser);
    return <>
        { auth.currentUser == null
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
