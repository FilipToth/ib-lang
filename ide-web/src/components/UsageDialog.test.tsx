const mockGetLimits = jest.fn();

jest.mock("services/server", () => ({
    getLimits: () => mockGetLimits(),
}));

import { render, screen, waitFor } from "@testing-library/react";
import UsageDialog from "./UsageDialog";

const limits = {
    files: { used: 3, allowed: 64 },
    bytes: { used: 4096, allowed: 16 * 1024 * 1024 },
    bytes_per_file: 256 * 1024,
    runs: { used: 2, allowed: 12 },
    run_window_seconds: 60,
    steps_per_run: 50_000_000,
    elements_per_run: 2_000_000,
};

describe("the usage dialog", () => {
    beforeEach(() => mockGetLimits.mockReset());

    it("shows what is used against what is allowed", async () => {
        mockGetLimits.mockResolvedValue(limits);
        render(<UsageDialog open={true} onClose={() => {}} />);

        expect(await screen.findByText("3 of 64")).toBeInTheDocument();
        expect(screen.getByText("4 KB of 16.0 MB")).toBeInTheDocument();
        expect(screen.getByText("2 of 12")).toBeInTheDocument();
    });

    /// The per-run caps are a ceiling every run starts from, so they are stated
    /// rather than drawn as something being filled up.
    it("states the per-run caps", async () => {
        mockGetLimits.mockResolvedValue(limits);
        render(<UsageDialog open={true} onClose={() => {}} />);

        expect(await screen.findByText(/50,000,000 steps/)).toBeInTheDocument();
        expect(screen.getByText(/2,000,000 items/)).toBeInTheDocument();
    });

    it("says so when the usage cannot be loaded", async () => {
        mockGetLimits.mockRejectedValue(new Error("offline"));
        render(<UsageDialog open={true} onClose={() => {}} />);

        expect(
            await screen.findByText(/Could not load your usage/),
        ).toBeInTheDocument();
    });

    /// Nothing is asked for until it is looked at.
    it("does not fetch while closed", () => {
        mockGetLimits.mockResolvedValue(limits);
        render(<UsageDialog open={false} onClose={() => {}} />);

        expect(mockGetLimits).not.toHaveBeenCalled();
    });

    it("re-reads every time it opens, since the allowances move", async () => {
        mockGetLimits.mockResolvedValue(limits);
        const { rerender } = render(
            <UsageDialog open={true} onClose={() => {}} />,
        );

        await waitFor(() => expect(mockGetLimits).toHaveBeenCalledTimes(1));

        rerender(<UsageDialog open={false} onClose={() => {}} />);
        rerender(<UsageDialog open={true} onClose={() => {}} />);

        await waitFor(() => expect(mockGetLimits).toHaveBeenCalledTimes(2));
    });
});
