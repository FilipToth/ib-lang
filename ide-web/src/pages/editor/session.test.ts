import { IBFile } from "services/server";
import { Pane, openTab, singlePane } from "./panes";
import { StoredSession, restoreSession, sessionOf } from "./session";

const file = (id: string): IBFile => ({
    id,
    filename: `${id}.ib`,
    contents: `contents of ${id}`,
});

const files = [file("a"), file("b"), file("c")];

/// The tab names in each pane, and which of them is open, as "a [b] c".
const layout = (panes: Pane[]) =>
    panes
        .map((pane) =>
            pane.tabs
                .map((tab, index) => {
                    const name =
                        tab.kind == "file" ? tab.file.filename : tab.title;
                    return index == pane.active ? `[${name}]` : name;
                })
                .join(" "),
        )
        .join(" | ");

/// A session as the editor would store it: named tabs per pane, the last of
/// each open.
const stored = (...sides: string[][]): StoredSession => ({
    panes: sides.map((names) => ({
        active: names.length - 1,
        tabs: names.map((name) =>
            name.startsWith("graph:")
                ? { kind: "graph" as const, fileId: name.slice(6) }
                : { kind: "file" as const, fileId: name },
        ),
    })),
    focused: 0,
});

describe("restoring the session", () => {
    it("reopens the tabs of both panes", () => {
        const session = restoreSession(stored(["a", "b"], ["graph:a"]), files);

        expect(layout(session.panes)).toBe("a.ib [b.ib] | [a.ib flow]");
    });

    it("keeps the pane that was in use", () => {
        const from = { ...stored(["a"], ["b"]), focused: 1 };

        expect(restoreSession(from, files).focused).toBe(1);
    });

    /// The file may have been renamed, or its contents changed, on another
    /// device since; tabs are kept by id and filled from the files as loaded.
    it("takes names and contents from the files as they are now", () => {
        const renamed = [{ ...file("a"), filename: "solver.ib" }];
        const session = restoreSession(stored(["a", "graph:a"]), renamed);

        expect(layout(session.panes)).toBe("solver.ib [solver.ib flow]");
    });

    it("leaves out tabs whose file is gone", () => {
        const session = restoreSession(
            stored(["a", "deleted"], ["graph:deleted"]),
            files,
        );

        expect(layout(session.panes)).toBe("[a.ib]");
    });

    it("opens the first file when there is nothing to reopen", () => {
        expect(layout(restoreSession(null, files).panes)).toBe("[a.ib]");
        expect(layout(restoreSession(stored([]), files).panes)).toBe("[a.ib]");
    });

    it("leaves an empty workspace when there are no files", () => {
        const session = restoreSession(stored(["a"]), []);

        expect(session.panes).toEqual(singlePane());
    });

    /// Stored sessions are whatever the browser holds, so nonsense in them
    /// must not leave the editor pointing at a tab that is not there.
    it("holds the open tab and the pane in range", () => {
        const session = restoreSession(
            {
                panes: [{ tabs: [{ kind: "file", fileId: "a" }], active: 7 }],
                focused: 5,
            },
            files,
        );

        expect(layout(session.panes)).toBe("[a.ib]");
        expect(session.focused).toBe(0);
    });

    it("opens a file once, however often it was stored", () => {
        const session = restoreSession(stored(["a", "a"], ["a"]), files);

        expect(layout(session.panes)).toBe("[a.ib]");
    });

    it("stores what it restores", () => {
        const panes = openTab(
            openTab(singlePane(), 0, { kind: "file", file: files[0] }),
            1,
            { kind: "file", file: files[1] },
        );

        const session = restoreSession(sessionOf(panes, 1), files);

        expect(layout(session.panes)).toBe(layout(panes));
        expect(session.focused).toBe(1);
    });
});
