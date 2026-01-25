import { createTheme, useColorScheme } from "@mui/material/styles";
import { red } from "@mui/material/colors";

/// The colours both the interface and the code editor are drawn from, so the
/// two read as one surface in either mode. The sidebar is a shade off that
/// surface, so the file list stands apart from the code beside it.
export const surfaces = {
    light: {
        background: "#ffffff",
        paper: "#ffffff",
        sidebar: "#f3f4f7",
        text: "#1f2328",
    },
    dark: {
        background: "#0f1117",
        paper: "#161a23",
        sidebar: "#161a23",
        text: "#e6e8ee",
    },
};

const theme = createTheme({
    cssVariables: {
        // a class on <html>, rather than the system setting alone, so the
        // toggle can override it; public/index.html sets it before the first
        // paint too
        colorSchemeSelector: "class",
    },
    colorSchemes: {
        light: {
            palette: {
                primary: { main: "#1565c0" },
                secondary: { main: "#19857b" },
                error: { main: red.A400 },
                background: {
                    default: surfaces.light.background,
                    paper: surfaces.light.paper,
                },
                text: { primary: surfaces.light.text },
            },
        },
        dark: {
            palette: {
                // the light primary is too dark to read on the dark surface
                primary: { main: "#64b5f6" },
                secondary: { main: "#4db6ac" },
                error: { main: "#ff6e6e" },
                background: {
                    default: surfaces.dark.background,
                    paper: surfaces.dark.paper,
                },
                text: { primary: surfaces.dark.text },
            },
        },
    },
});

export default theme;

/// The mode the page is shown in, with "system" resolved to what the system
/// prefers.
export const useResolvedMode = (): "light" | "dark" => {
    const { mode, systemMode } = useColorScheme();
    const resolved = mode == "system" ? systemMode : mode;

    // not known until the provider has read the setting; the script in
    // public/index.html has already put it on the page by then
    if (resolved == null) {
        return document.documentElement.classList.contains("dark")
            ? "dark"
            : "light";
    }

    return resolved;
};
