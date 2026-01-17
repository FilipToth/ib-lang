/// Where a dragged tab would land: beside the tab at `index`, on `side`.
export interface DropTarget {
    index: number;
    side: "before" | "after";
}

/// The index the tab at `from` ends up at when dropped on `target`.
///
/// The slot is counted before the tab is taken out of the list, so a slot
/// after it shifts left once it is.
export const dropIndex = (from: number, target: DropTarget): number => {
    let to = target.index + (target.side == "after" ? 1 : 0);
    if (from < to) to -= 1;

    return to;
};

/// A copy of `items` with the one at `from` moved to `to`.
export const moveItem = <T>(items: T[], from: number, to: number): T[] => {
    const moved = [...items];
    const [item] = moved.splice(from, 1);
    moved.splice(to, 0, item);

    return moved;
};
