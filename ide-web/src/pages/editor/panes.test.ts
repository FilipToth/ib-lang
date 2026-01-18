import { IBFile } from "services/server";
import {
    EditorTab,
    Pane,
    closeTab,
    findTab,
    moveTab,
    openTab,
    singlePane,
} from "./panes";

const file = (name: string): EditorTab => ({
    kind: "file",
    file: { id: name, filename: `${name}.ib`, contents: "" } as IBFile,
});

/// The tab names in each pane, and which of them is open, as "a [b] c".
const layout = (panes: Pane[]) =>
    panes
        .map((pane) =>
            pane.tabs
                .map((tab, index) => {
                    const name = tab.kind == "file" ? tab.file.id : tab.id;
                    return index == pane.active ? `[${name}]` : name;
                })
                .join(" ")
        )
        .join(" | ");

/// Panes holding the named tabs, the first of each open.
const panesOf = (...sides: string[][]) =>
    sides.reduce(
        (panes, names, side) =>
            names.reduce((acc, name) => openTab(acc, side, file(name)), panes),
        singlePane()
    );

describe("panes", () => {
    it("opens a tab in the pane it is given, making the second one", () => {
        let panes = panesOf(["a"]);
        panes = openTab(panes, 1, file("graph"));

        expect(layout(panes)).toBe("[a] | [graph]");
    });

    it("opens further tabs beside the ones already there", () => {
        const panes = panesOf(["a", "b"], ["c"]);
        expect(layout(panes)).toBe("a [b] | [c]");
    });

    describe("closing", () => {
        it("falls back to the tab on the left", () => {
            const panes = panesOf(["a", "b", "c"]);
            expect(layout(closeTab(panes, { pane: 0, index: 2 }))).toBe(
                "a [b]"
            );
        });

        it("keeps the open tab open when one before it closes", () => {
            const panes = panesOf(["a", "b", "c"]);
            expect(layout(closeTab(panes, { pane: 0, index: 0 }))).toBe(
                "b [c]"
            );
        });

        /// A pane is only there while it holds something; the other one then
        /// takes the whole width.
        it("drops a pane left without tabs", () => {
            const panes = panesOf(["a"], ["graph"]);
            expect(layout(closeTab(panes, { pane: 1, index: 0 }))).toBe("[a]");
        });

        it("drops the first pane too, leaving the second one", () => {
            const panes = panesOf(["a"], ["graph"]);
            expect(layout(closeTab(panes, { pane: 0, index: 0 }))).toBe(
                "[graph]"
            );
        });

        it("leaves one empty pane when the last tab closes", () => {
            const panes = closeTab(panesOf(["a"]), { pane: 0, index: 0 });
            expect(panes).toHaveLength(1);
            expect(panes[0].tabs).toHaveLength(0);
        });
    });

    describe("moving", () => {
        it("reorders within a pane, keeping the open tab open", () => {
            const panes = panesOf(["a", "b", "c"]);
            const moved = moveTab(
                panes,
                { pane: 0, index: 0 },
                0,
                { index: 2, side: "after" }
            );

            expect(layout(moved.panes)).toBe("b [c] a");
        });

        it("moves a tab to the other pane and opens it there", () => {
            const panes = panesOf(["a", "b"], ["graph"]);
            const moved = moveTab(
                panes,
                { pane: 0, index: 1 },
                1,
                { index: 0, side: "before" }
            );

            expect(layout(moved.panes)).toBe("[a] | [b] graph");
            expect(moved.focus).toEqual({ pane: 1, index: 0 });
        });

        /// Dragging a graph out of the split and onto the other side leaves
        /// it as an ordinary full-width tab.
        it("drops the pane the moved tab came from when it empties", () => {
            const panes = panesOf(["a"], ["graph"]);
            const moved = moveTab(
                panes,
                { pane: 1, index: 0 },
                0,
                { index: 0, side: "after" }
            );

            expect(layout(moved.panes)).toBe("a [graph]");
            expect(moved.focus).toEqual({ pane: 0, index: 1 });
        });

        it("puts a tab dropped beside the tabs at the end", () => {
            const panes = panesOf(["a", "b"], ["graph"]);
            const moved = moveTab(panes, { pane: 0, index: 0 }, 1, null);

            expect(layout(moved.panes)).toBe("[b] | graph [a]");
        });

        it("makes a second pane for a tab moved into one", () => {
            const panes = panesOf(["a", "b"]);
            const moved = moveTab(panes, { pane: 0, index: 0 }, 1, null);

            expect(layout(moved.panes)).toBe("[b] | [a]");
        });

        it("leaves a tab dropped where it already is", () => {
            const panes = panesOf(["a", "b"]);
            const moved = moveTab(
                panes,
                { pane: 0, index: 0 },
                0,
                { index: 0, side: "after" }
            );

            expect(layout(moved.panes)).toBe("a [b]");
        });
    });

    it("finds a tab in either pane", () => {
        const panes = panesOf(["a"], ["b"]);
        const found = findTab(
            panes,
            (tab) => tab.kind == "file" && tab.file.id == "b"
        );

        expect(found).toEqual({ pane: 1, index: 0 });
    });
});
