import { IconButton, Tooltip } from "@mui/material";
import { DarkModeRounded, LightModeRounded } from "@mui/icons-material";
import { useColorScheme } from "@mui/material/styles";
import { useResolvedMode } from "theme";

/// Switches between light and dark mode. Until it is first used the page
/// follows the system setting; after that the choice is remembered.
const ThemeToggle = () => {
    const { setMode } = useColorScheme();
    const mode = useResolvedMode();
    const next = mode == "dark" ? "light" : "dark";

    return (
        <Tooltip title={`Switch to ${next} mode`}>
            <IconButton
                size="large"
                color="inherit"
                aria-label={`Switch to ${next} mode`}
                onClick={() => setMode(next)}
            >
                {mode == "dark" ? <LightModeRounded /> : <DarkModeRounded />}
            </IconButton>
        </Tooltip>
    );
};

export default ThemeToggle;
