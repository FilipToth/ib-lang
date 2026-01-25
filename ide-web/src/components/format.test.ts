import { formatBytes, formatCount } from "./format";

describe("formatBytes", () => {
    it("scales to the unit that reads", () => {
        expect(formatBytes(0)).toBe("0 B");
        expect(formatBytes(512)).toBe("512 B");
        expect(formatBytes(1024)).toBe("1 KB");
        expect(formatBytes(256 * 1024)).toBe("256 KB");
        expect(formatBytes(16 * 1024 * 1024)).toBe("16.0 MB");
    });
});

describe("formatCount", () => {
    /// The budgets run to eight digits, which are unreadable ungrouped.
    it("groups digits", () => {
        expect(formatCount(0)).toBe("0");
        expect(formatCount(999)).toBe("999");
        expect(formatCount(2_000_000)).toBe("2,000,000");
        expect(formatCount(50_000_000)).toBe("50,000,000");
    });
});
