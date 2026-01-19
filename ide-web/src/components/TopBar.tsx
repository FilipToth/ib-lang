import {
    AppBar,
    IconButton,
    Toolbar,
    Menu,
    MenuItem,
    Box,
} from "@mui/material";
import { AccountCircle } from "@mui/icons-material";
import React, { ReactNode, useState } from "react";
import { auth } from "services/firebase";
import { useNavigate } from "react-router-dom";
import ThemeToggle from "./ThemeToggle";

export const TopBar = ({ children }: { children: ReactNode }) => {
    const navigate = useNavigate();
    const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

    const accountClick = (e: React.MouseEvent<HTMLElement>) => {
        setAnchorEl(e.currentTarget);
    };

    const closeMenu = () => {
        setAnchorEl(null);
    };

    const signOut = () => {
        auth.signOut();
        navigate("/login");
    };

    return (
        <AppBar position="static">
            <Toolbar>
                {children}
                <Box sx={{ flexGrow: 1 }}></Box>
                <ThemeToggle />
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
