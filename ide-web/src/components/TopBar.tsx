import {
    AppBar,
    IconButton,
    Toolbar,
    Typography,
    Menu,
    MenuItem,
} from "@mui/material";
import { AccountCircle, MenuRounded } from "@mui/icons-material";
import React, { useState } from "react";
import { auth } from "services/firebase";
import { useNavigate } from "react-router-dom";

export const TopBar = () => {
    const navigate = useNavigate();
    const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

    const accountClick = (e: React.MouseEvent<HTMLElement>) => {
        setAnchorEl(e.currentTarget);
    };

    const closeMenu = () => {
        setAnchorEl(null);
    };

    const signOut = () => {
        console.log(auth.currentUser);
        auth.signOut();
        navigate("/login");
    };

    return (
        <AppBar position="static">
            <Toolbar>
                <IconButton size="large">
                    <MenuRounded />
                </IconButton>
                <Typography variant="h6" sx={{ flexGrow: 1 }}>
                    Code Editor
                </Typography>
                <div>
                    <IconButton
                        size="large"
                        aria-haspopup={false}
                        aria-controls="menu-appbar"
                        onClick={accountClick}
                    >
                        <AccountCircle />
                    </IconButton>

                    <Menu
                        id="menu-appbar"
                        keepMounted
                        open={Boolean(anchorEl)}
                        anchorEl={anchorEl}
                        onClose={closeMenu}
                    >
                        <MenuItem onClick={signOut}>Sign Out</MenuItem>
                    </Menu>
                </div>
            </Toolbar>
        </AppBar>
    );
};
