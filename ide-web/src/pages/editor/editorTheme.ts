import { tags as t } from "@lezer/highlight";
import { createTheme } from "thememirror";
import { surfaces } from "theme";

/// The code editor's look in light mode. Its background is the interface's,
/// and the keyword colour is a shade of the primary one, so the editor reads
/// as part of the page rather than a panel set into it.
export const lightEditorTheme = createTheme({
    variant: "light",
    settings: {
        background: surfaces.light.background,
        foreground: surfaces.light.text,
        caret: surfaces.light.text,
        selection: "#d7dcf7",
        lineHighlight: "#f3f4f8",
        gutterBackground: surfaces.light.background,
        gutterForeground: "#8c8f98",
    },
    styles: [
        { tag: t.lineComment, color: "#6e7781", fontStyle: "italic" },
        { tag: t.keyword, color: "#3f51b5", fontWeight: "600" },
        { tag: t.operator, color: "#00838f" },
        { tag: t.string, color: "#2e7d32" },
        { tag: [t.number, t.bool], color: "#c75400" },
        { tag: t.function(t.variableName), color: "#00695c" },
        { tag: t.typeName, color: "#ad1457" },
        { tag: t.variableName, color: surfaces.light.text },
        { tag: [t.paren, t.squareBracket], color: "#57606a" },
    ],
});

/// The same, in dark mode.
export const darkEditorTheme = createTheme({
    variant: "dark",
    settings: {
        background: surfaces.dark.background,
        foreground: surfaces.dark.text,
        caret: surfaces.dark.text,
        selection: "#2f3a66",
        lineHighlight: "#ffffff0a",
        gutterBackground: surfaces.dark.background,
        gutterForeground: "#6b7080",
    },
    styles: [
        { tag: t.lineComment, color: "#7f8490", fontStyle: "italic" },
        { tag: t.keyword, color: "#8c9eff", fontWeight: "600" },
        { tag: t.operator, color: "#80deea" },
        { tag: t.string, color: "#a5d6a7" },
        { tag: [t.number, t.bool], color: "#ffcc80" },
        { tag: t.function(t.variableName), color: "#80cbc4" },
        { tag: t.typeName, color: "#f48fb1" },
        { tag: t.variableName, color: surfaces.dark.text },
        { tag: [t.paren, t.squareBracket], color: "#9aa0ad" },
    ],
});
