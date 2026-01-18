import { fireEvent, render, screen } from "@testing-library/react";
import Splitter from "./Splitter";

// jsdom has no PointerEvent, and the plain Event it falls back to drops the
// pointer's position
if (typeof window.PointerEvent == "undefined") {
    class PointerEvent extends MouseEvent {
        pointerId: number;

        constructor(type: string, init: PointerEventInit = {}) {
            super(type, init);
            this.pointerId = init.pointerId ?? 0;
        }
    }

    (window as any).PointerEvent = PointerEvent;
}

/// A splitter after a pane `before` pixels wide, resizing one `width` wide.
function setup(width: number, before: number) {
    const setWidth = jest.fn();

    render(
        <div>
            <div data-testid="before" />
            <Splitter
                width={width}
                setWidth={setWidth}
                minWidth={200}
                minBefore={300}
            />
        </div>
    );

    // jsdom does no layout
    const pane = screen.getByTestId("before");
    pane.getBoundingClientRect = () => ({ width: before } as DOMRect);

    const splitter = screen.getByRole("separator");
    // jsdom has no pointer capture
    splitter.setPointerCapture = () => {};

    return { splitter, setWidth };
}

describe("Splitter", () => {
    it("widens the pane after it when dragged left", () => {
        const { splitter, setWidth } = setup(400, 800);

        fireEvent.pointerDown(splitter, { clientX: 500, pointerId: 1 });
        fireEvent.pointerMove(splitter, { clientX: 450, pointerId: 1 });

        expect(setWidth).toHaveBeenLastCalledWith(450);
    });

    it("narrows it when dragged right, down to its minimum", () => {
        const { splitter, setWidth } = setup(400, 800);

        fireEvent.pointerDown(splitter, { clientX: 500, pointerId: 1 });
        fireEvent.pointerMove(splitter, { clientX: 550, pointerId: 1 });
        expect(setWidth).toHaveBeenLastCalledWith(350);

        fireEvent.pointerMove(splitter, { clientX: 900, pointerId: 1 });
        expect(setWidth).toHaveBeenLastCalledWith(200);
    });

    /// The pane before gives up room only down to its own minimum.
    it("leaves the pane before it its minimum", () => {
        const { splitter, setWidth } = setup(400, 800);

        fireEvent.pointerDown(splitter, { clientX: 500, pointerId: 1 });
        fireEvent.pointerMove(splitter, { clientX: 0, pointerId: 1 });

        expect(setWidth).toHaveBeenLastCalledWith(400 + 800 - 300);
    });

    it("stops following the pointer once released", () => {
        const { splitter, setWidth } = setup(400, 800);

        fireEvent.pointerDown(splitter, { clientX: 500, pointerId: 1 });
        fireEvent.pointerUp(splitter, { clientX: 500, pointerId: 1 });
        fireEvent.pointerMove(splitter, { clientX: 450, pointerId: 1 });

        expect(setWidth).not.toHaveBeenCalled();
    });

    it("moves with the arrow keys", () => {
        const { splitter, setWidth } = setup(400, 800);

        fireEvent.keyDown(splitter, { key: "ArrowLeft" });
        expect(setWidth).toHaveBeenLastCalledWith(416);

        fireEvent.keyDown(splitter, { key: "ArrowRight" });
        expect(setWidth).toHaveBeenLastCalledWith(384);
    });
});
