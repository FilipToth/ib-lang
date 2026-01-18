import { IBFile } from "services/server";
import { DropTarget, dropIndex, moveItem } from "./tabOrder";

/// An open tab. Files are edited; graphs are drawn from the DOT the analyzer
/// produces, and are read-only.
export interface GraphTab {
    kind: "graph";
    id: string;
    title: string;
    fileId: string | null;
}

export type EditorTab = { kind: "file"; file: IBFile } | GraphTab;

export const tabId = (tab: EditorTab) =>
    tab.kind == "file" ? tab.file.id : tab.id;

export const tabTitle = (tab: EditorTab) =>
    tab.kind == "file" ? tab.file.filename : tab.title;

/// The graph tab of `file`. Its id is what tells one graph tab from another,
/// so it is made here rather than at each place one is opened.
export const graphTab = (file: IBFile | null): GraphTab => ({
    kind: "graph",
    id: file == null ? "graph" : `graph:${file.id}`,
    title: file == null ? "Control Flow" : `${file.filename} flow`,
    fileId: file?.id ?? null,
});

/// One side of the editor: its own tabs, one of them open.
export interface Pane {
    tabs: EditorTab[];
    active: number;
}

/// Which tab, in which pane.
export interface TabRef {
    pane: number;
    index: number;
}

/// The editor is one pane, or two side by side. A pane that loses its last
/// tab goes away, so a second pane exists only while it holds something.
export const singlePane = (): Pane[] => [{ tabs: [], active: 0 }];

export const activeTab = (pane: Pane | undefined): EditorTab | undefined =>
    pane?.tabs[pane.active];

export const tabAt = (panes: Pane[], ref: TabRef): EditorTab | undefined =>
    panes[ref.pane]?.tabs[ref.index];

/// Where the first tab `matches` is, if it is open at all.
export const findTab = (
    panes: Pane[],
    matches: (tab: EditorTab) => boolean
): TabRef | null => {
    for (let pane = 0; pane < panes.length; pane++) {
        const index = panes[pane].tabs.findIndex(matches);
        if (index != -1) return { pane, index };
    }

    return null;
};

export const eachTab = (panes: Pane[]): EditorTab[] =>
    panes.flatMap((pane) => pane.tabs);

/// Opens `tab` in pane `pane`, which is made if it is the second one, and
/// makes it the pane's open tab.
export const openTab = (
    panes: Pane[],
    pane: number,
    tab: EditorTab
): Pane[] => {
    const next = [...panes];

    // only ever one more than there are: panes are the two sides, not a list
    if (pane >= next.length) next.push({ tabs: [], active: 0 });

    const tabs = [...next[pane].tabs, tab];
    next[pane] = { tabs, active: tabs.length - 1 };

    return next;
};

export const focusTab = (panes: Pane[], ref: TabRef): Pane[] =>
    panes.map((pane, index) =>
        index == ref.pane ? { ...pane, active: ref.index } : pane
    );

/// The tab that takes over from the one closed at `closed`.
const activeAfterClose = (active: number, closed: number, left: number) => {
    // one before the open tab shifts it left; the open one falls back to its
    // neighbour on the left
    const next =
        closed < active
            ? active - 1
            : closed == active
            ? closed - 1
            : active;

    return Math.max(0, Math.min(next, left - 1));
};

/// A pane without tabs is not shown, so the other one takes the whole width.
const collapse = (panes: Pane[]): Pane[] => {
    if (panes.length < 2) return panes;

    const kept = panes.filter((pane) => pane.tabs.length > 0);
    if (kept.length == panes.length) return panes;

    return kept.length == 0 ? singlePane() : kept;
};

export const closeTab = (panes: Pane[], ref: TabRef): Pane[] => {
    const pane = panes[ref.pane];
    if (pane == null || pane.tabs[ref.index] == null) return panes;

    const tabs = pane.tabs.filter((_, index) => index != ref.index);
    const next = [...panes];
    next[ref.pane] = {
        tabs,
        active: activeAfterClose(pane.active, ref.index, tabs.length),
    };

    return collapse(next);
};

/// Moves a tab within its pane or into the other one.
///
/// `target` is the tab it was dropped on, and which side of it. The moved tab
/// is the open one wherever it lands, and `focus` says where that is: panes
/// shift when one is left empty and goes away.
export const moveTab = (
    panes: Pane[],
    from: TabRef,
    to: number,
    target: DropTarget | null
): { panes: Pane[]; focus: TabRef } => {
    const moved = tabAt(panes, from);
    const unchanged = { panes, focus: from };
    if (moved == null) return unchanged;

    if (from.pane == to) {
        if (target == null) return unchanged;

        const index = dropIndex(from.index, target);
        if (index == from.index) return unchanged;

        const pane = panes[to];
        const tabs = moveItem(pane.tabs, from.index, index);
        const next = [...panes];
        // the tab that was open stays open, wherever it now is
        next[to] = { tabs, active: tabs.indexOf(activeTab(pane)!) };

        return { panes: next, focus: { pane: to, index } };
    }

    // dropped on the far side of the last tab, or on the empty space beside
    // the tabs, it goes to the end
    const into = panes[to]?.tabs ?? [];
    const index =
        target == null
            ? into.length
            : target.index + (target.side == "after" ? 1 : 0);

    let next = openTab(panes, to, moved);
    const tabs = [...next[to].tabs];
    tabs.splice(index, 0, tabs.pop()!);
    next[to] = { tabs, active: index };

    next = closeTab(next, from);

    const focus = findTab(next, (tab) => tab == moved);
    return { panes: next, focus: focus ?? { pane: 0, index: 0 } };
};
