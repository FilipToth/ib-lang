import { render, screen } from "@testing-library/react";
import OutputView from "./OutputView";

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

describe("OutputView", () => {
    it("shows output as printed, line breaks and spacing kept", () => {
        render(<OutputView output={"1\n  2\n"} />);

        expect(screen.getByTestId("output").textContent).toBe("1\n  2\n");
    });

    it("follows new output to the bottom", () => {
        const { rerender } = render(<OutputView output="a" />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        rerender(<OutputView output={"a\nb"} />);

        expect(panel.scrollTop).toBe(500);
    });

    /// Someone reading earlier output keeps their place as more arrives.
    it("stays put once scrolled up", () => {
        const { rerender } = render(<OutputView output="a" />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        panel.scrollTop = 50;
        panel.dispatchEvent(new Event("scroll"));

        sized(panel, 800, 100);
        rerender(<OutputView output={"a\nb"} />);

        expect(panel.scrollTop).toBe(50);
    });

    it("follows again after a new run clears the output", () => {
        const { rerender } = render(<OutputView output="a" />);
        const panel = screen.getByTestId("output");

        sized(panel, 500, 100);
        panel.scrollTop = 50;
        panel.dispatchEvent(new Event("scroll"));

        rerender(<OutputView output="" />);
        sized(panel, 900, 100);
        rerender(<OutputView output="new run" />);

        expect(panel.scrollTop).toBe(900);
    });
});
