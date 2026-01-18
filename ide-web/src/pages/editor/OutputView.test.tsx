import { render, screen } from "@testing-library/react";
import OutputView, { OutputEntry, appendEntry } from "./OutputView";

/// jsdom does no layout, so the panel is given the sizes a browser would.
function sized(panel: HTMLElement, scrollHeight: number, clientHeight: number) {
    Object.defineProperty(panel, "scrollHeight", {
        configurable: true,
        value: scrollHeight,
    });
    Object.defineProperty(panel, "clientHeight", {
        configurable: true,
        value: clientHeight,
    });
}

const printed = (text: string): OutputEntry => ({ kind: "output", text });
const failed = (text: string): OutputEntry => ({ kind: "error", text });

describe("OutputView", () => {
    it("shows output as printed, line breaks and spacing kept", () => {
        render(<OutputView entries={[printed("1\n  2\n")]} />);

        expect(screen.getByTestId("output").textContent).toBe("1\n  2\n");
    });

    /// What went wrong is marked apart from what the program printed, and
    /// drawn in the error colour.
    it("marks what went wrong", () => {
        render(
            <OutputView
                entries={[printed("1\n"), failed("Runtime error: /0\n")]}
            />
        );

        const panel = screen.getByTestId("output");
        expect(panel.textContent).toBe("1\nRuntime error: /0\n");

        const errors = panel.querySelectorAll('[data-kind="error"]');
        expect(errors).toHaveLength(1);
        expect(errors[0].textContent).toBe("Runtime error: /0\n");
    });

    it("follows new output to the bottom", () => {
        const { rerender } = render(<OutputView entries={[printed("a")]} />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        rerender(<OutputView entries={[printed("a\nb")]} />);

        expect(panel.scrollTop).toBe(500);
    });

    /// Someone reading earlier output keeps their place as more arrives.
    it("stays put once scrolled up", () => {
        const { rerender } = render(<OutputView entries={[printed("a")]} />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        panel.scrollTop = 50;
        panel.dispatchEvent(new Event("scroll"));

        sized(panel, 800, 100);
        rerender(<OutputView entries={[printed("a\nb")]} />);

        expect(panel.scrollTop).toBe(50);
    });

    it("follows again after a new run clears the output", () => {
        const { rerender } = render(<OutputView entries={[printed("a")]} />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        panel.scrollTop = 50;
        panel.dispatchEvent(new Event("scroll"));

        rerender(<OutputView entries={[]} />);
        sized(panel, 900, 100);
        rerender(<OutputView entries={[printed("new run")]} />);

        expect(panel.scrollTop).toBe(900);
    });

    describe("appendEntry", () => {
        it("joins text of the kind last added", () => {
            const entries = appendEntry(
                appendEntry([], "output", "1\n"),
                "output",
                "2\n"
            );

            expect(entries).toEqual([printed("1\n2\n")]);
        });

        it("starts a piece of its own for the other kind", () => {
            const entries = appendEntry(
                appendEntry([], "output", "1\n"),
                "error",
                "stopped\n"
            );

            expect(entries).toEqual([printed("1\n"), failed("stopped\n")]);
        });

        it("does not change the entries it is given", () => {
            const entries = [printed("1\n")];
            appendEntry(entries, "output", "2\n");

            expect(entries).toEqual([printed("1\n")]);
        });
    });
});
