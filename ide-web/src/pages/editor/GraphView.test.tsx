// the real module reaches axios and firebase, neither of which jest can
// transform, and the component only calls this one function
const mockGetControlFlowGraph = jest.fn();

jest.mock("services/server", () => ({
    getControlFlowGraph: (code: string) => mockGetControlFlowGraph(code),
}));

import { act, render, screen, waitFor } from "@testing-library/react";
import GraphView from "./GraphView";

describe("GraphView", () => {
    beforeEach(() => mockGetControlFlowGraph.mockReset());

    /// A program that does not compile has no graph. The reason has to reach
    /// the reader instead of an empty panel, and drawing must not be attempted.
    it("reports why a graph could not be drawn", async () => {
        mockGetControlFlowGraph.mockResolvedValue({
            dot: null,
            diagnostics: [
                { message: "Cannot find value 'X'", offset_start: 0, offset_end: 1 },
            ],
        });

        render(<GraphView code={"output X"} />);

        await waitFor(() => {
            expect(
                screen.getByText(/Cannot find value 'X'/)
            ).toBeInTheDocument();
        });
    });

    it("falls back to a general reason when no diagnostic came back", async () => {
        mockGetControlFlowGraph.mockResolvedValue({ dot: null, diagnostics: [] });

        render(<GraphView code={"output X"} />);

        await waitFor(() => {
            expect(
                screen.getByText(/does not compile/)
            ).toBeInTheDocument();
        });
    });

    it("surfaces a failed request", async () => {
        mockGetControlFlowGraph.mockRejectedValue(new Error("Network Error"));

        render(<GraphView code={"output 1"} />);

        await waitFor(() => {
            expect(screen.getByText(/Network Error/)).toBeInTheDocument();
        });
    });

    /// The drawing follows the code, but a redraw is a round trip and a
    /// graphviz layout, so it waits for the edits to pause.
    it("redraws once edits pause, not on every change", async () => {
        jest.useFakeTimers();
        mockGetControlFlowGraph.mockResolvedValue({
            dot: null,
            diagnostics: [],
        });

        const { rerender } = render(<GraphView code={"output 1"} />);
        await act(async () => {});
        expect(mockGetControlFlowGraph).toHaveBeenCalledTimes(1);

        rerender(<GraphView code={"output 12"} />);
        await act(async () => {
            jest.advanceTimersByTime(400);
        });
        rerender(<GraphView code={"output 123"} />);
        await act(async () => {
            jest.advanceTimersByTime(700);
        });
        expect(mockGetControlFlowGraph).toHaveBeenCalledTimes(1);

        await act(async () => {
            jest.advanceTimersByTime(100);
        });
        expect(mockGetControlFlowGraph).toHaveBeenCalledTimes(2);
        expect(mockGetControlFlowGraph).toHaveBeenLastCalledWith("output 123");

        jest.useRealTimers();
    });

    it("asks the server for the code it was given", async () => {
        mockGetControlFlowGraph.mockResolvedValue({ dot: null, diagnostics: [] });

        render(<GraphView code={"output 42"} />);

        await waitFor(() => {
            expect(mockGetControlFlowGraph).toHaveBeenCalledWith("output 42");
        });
    });
});
