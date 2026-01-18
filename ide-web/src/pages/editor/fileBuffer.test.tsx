import { StrictMode } from "react";
import { renderHook } from "@testing-library/react";
import { IBFile } from "services/server";
import { useFileBuffer } from "./fileBuffer";

const file = (id: string, contents: string): IBFile => ({
    id,
    filename: `${id}.ib`,
    contents,
});

describe("useFileBuffer", () => {
    /// Strict mode, as the app runs it: React renders twice and keeps the
    /// second run, which a version of this that wrote a ref during the first
    /// run did not survive -- switching tabs kept the previous file's code.
    const render = (initial: IBFile | null) =>
        renderHook((props: IBFile | null) => useFileBuffer(props), {
            wrapper: StrictMode,
            initialProps: initial,
        });

    it("holds the file it is given", () => {
        const { result } = render(file("a", "output 1"));
        expect(result.current[0]).toBe("output 1");
    });

    it("fills from the other file when the tab changes", () => {
        const { result, rerender } = render(file("a", "output 1"));

        rerender(file("b", "output 2"));
        expect(result.current[0]).toBe("output 2");

        rerender(file("a", "output 1"));
        expect(result.current[0]).toBe("output 1");
    });

    /// Edits live in the buffer; the same file coming round again must not
    /// throw them away.
    it("keeps edits to the file it already holds", () => {
        const a = file("a", "output 1");
        const { result, rerender } = render(a);

        result.current[1]("output 1 edited");
        a.contents = "output 1 edited";
        rerender(a);

        expect(result.current[0]).toBe("output 1 edited");
    });

    /// A graph tab has no file, and going back to the file it was opened
    /// beside leaves the buffer as it was.
    it("keeps the buffer while no file is in front", () => {
        const a = file("a", "output 1");
        const { result, rerender } = render(a);

        rerender(null);
        expect(result.current[0]).toBe("output 1");

        rerender(a);
        expect(result.current[0]).toBe("output 1");
    });
});
