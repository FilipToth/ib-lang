import { dropIndex, moveItem } from "./tabOrder";

/// Where the tab at `from` lands in `tabs` when dropped on `target`.
const dropped = (
    tabs: string[],
    from: number,
    index: number,
    side: "before" | "after"
) => moveItem(tabs, from, dropIndex(from, { index, side })).join("");

describe("tab reordering", () => {
    const tabs = ["a", "b", "c", "d"];

    it("moves a tab right, past the one it is dropped after", () => {
        expect(dropped(tabs, 0, 2, "after")).toBe("bcad");
        expect(dropped(tabs, 0, 3, "after")).toBe("bcda");
    });

    it("moves a tab right, up to the one it is dropped before", () => {
        expect(dropped(tabs, 0, 2, "before")).toBe("bacd");
    });

    it("moves a tab left", () => {
        expect(dropped(tabs, 3, 0, "before")).toBe("dabc");
        expect(dropped(tabs, 3, 1, "after")).toBe("abdc");
    });

    /// Either side of the tab itself, or the near side of a neighbour, is
    /// where it already is.
    it("leaves a tab dropped next to itself in place", () => {
        expect(dropped(tabs, 1, 1, "before")).toBe("abcd");
        expect(dropped(tabs, 1, 1, "after")).toBe("abcd");
        expect(dropped(tabs, 1, 0, "after")).toBe("abcd");
        expect(dropped(tabs, 1, 2, "before")).toBe("abcd");
    });

    it("does not change the list it is given", () => {
        moveItem(tabs, 0, 3);
        expect(tabs.join("")).toBe("abcd");
    });
});
