import { IBFile } from "services/server";
import { EditorTab, Pane, graphTab, singlePane } from "./panes";

/// What is reopened after a reload. Tabs are kept by file id: names change,
/// contents come from the server, and a file that is gone takes its tabs with
/// it.
interface StoredTab {
    kind: "file" | "graph";
    fileId: string | null;
}

interface StoredPane {
    tabs: StoredTab[];
    active: number;
}

export interface StoredSession {
    panes: StoredPane[];
    focused: number;
}

const sessionKey = "ib.session";

export const sessionOf = (panes: Pane[], focused: number): StoredSession => ({
    panes: panes.map((pane) => ({
        active: pane.active,
        tabs: pane.tabs.map((tab) => ({
            kind: tab.kind,
            fileId: tab.kind == "file" ? tab.file.id : tab.fileId,
        })),
    })),
    focused,
});

/// The panes the stored session describes, as far as the files still allow.
///
/// Anything that no longer fits is left out rather than refused: a file may
/// have been deleted, or renamed, since the session was stored, and the
/// stored session itself is whatever was in the browser.
export const restoreSession = (
    stored: StoredSession | null,
    files: IBFile[]
): { panes: Pane[]; focused: number } => {
    const byId = new Map(files.map((file) => [file.id, file]));
    /// What is already open, since a file, or its graph, belongs to one tab.
    const taken = new Set<string>();

    const panes: Pane[] = [];

    for (const pane of stored?.panes ?? []) {
        const tabs: EditorTab[] = [];

        for (const tab of pane.tabs) {
            const file = tab.fileId == null ? null : byId.get(tab.fileId);
            if (file == null) continue;

            const key = `${tab.kind}:${file.id}`;
            if (taken.has(key)) continue;
            taken.add(key);

            tabs.push(
                tab.kind == "file" ? { kind: "file", file } : graphTab(file)
            );
        }

        if (tabs.length == 0) continue;

        panes.push({
            tabs,
            active: Math.max(0, Math.min(pane.active, tabs.length - 1)),
        });
    }

    if (panes.length == 0) {
        // nothing to reopen: start on the first file, rather than on the
        // empty workspace someone with files has no use for
        const first = files[0];
        return {
            panes:
                first == null
                    ? singlePane()
                    : [{ tabs: [{ kind: "file", file: first }], active: 0 }],
            focused: 0,
        };
    }

    const focused = Math.max(
        0,
        Math.min(stored?.focused ?? 0, panes.length - 1)
    );
    return { panes, focused };
};

/// The stored session, or null when there is none to make sense of. It is
/// whatever the browser holds, so it is read as unknown data.
export const loadStoredSession = (): StoredSession | null => {
    let raw: string | null = null;
    try {
        raw = localStorage.getItem(sessionKey);
    } catch {
        // storage can be off, which is the same as having nothing stored
    }

    if (raw == null) return null;

    try {
        const parsed = JSON.parse(raw);
        return isSession(parsed) ? parsed : null;
    } catch {
        return null;
    }
};

export const storeSession = (session: StoredSession) => {
    try {
        localStorage.setItem(sessionKey, JSON.stringify(session));
    } catch {
        // the session then lasts only as long as the page does
    }
};

const isSession = (value: unknown): value is StoredSession => {
    if (typeof value != "object" || value == null) return false;

    const session = value as StoredSession;
    if (typeof session.focused != "number") return false;
    if (!Array.isArray(session.panes)) return false;

    return session.panes.every(
        (pane) =>
            typeof pane?.active == "number" &&
            Array.isArray(pane.tabs) &&
            pane.tabs.every(
                (tab) =>
                    (tab?.kind == "file" || tab?.kind == "graph") &&
                    (tab.fileId == null || typeof tab.fileId == "string")
            )
    );
};
