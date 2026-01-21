// the socket and firebase are stood in for: this is about what the button does,
// not about reaching a server
const mockSendMessage = jest.fn();
let mockReadyState = 1;

jest.mock("react-use-websocket", () => ({
    __esModule: true,
    default: () => ({
        sendMessage: mockSendMessage,
        lastMessage: null,
        readyState: mockReadyState,
    }),
    ReadyState: {
        UNINSTANTIATED: -1,
        CONNECTING: 0,
        OPEN: 1,
        CLOSING: 2,
        CLOSED: 3,
    },
}));

jest.mock("services/firebase", () => ({ auth: { currentUser: null } }));

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import OutputBar from "./OutputBar";

const OPEN = 1;
const CLOSED = 3;

/// The kinds the server and client agree on, by position.
const STOP_KIND = 5;

function renderBar() {
    return render(
        <OutputBar
            code={"output 1"}
            fileId={"file-1"}
            filename={"a.ib"}
            onRuntimeError={() => {}}
            goTo={() => {}}
        />,
    );
}

describe("the run button", () => {
    beforeEach(() => {
        mockSendMessage.mockReset();
        mockReadyState = CLOSED;
    });

    it("runs the program when nothing is running", () => {
        renderBar();

        expect(screen.getByRole("button", { name: "Run program" })).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Stop program" })).toBeNull();
    });

    /// While a program runs there is nothing to run, and the one thing worth
    /// doing is stopping it.
    it("becomes a stop button once the program is running", () => {
        mockReadyState = OPEN;
        renderBar();

        expect(screen.getByRole("button", { name: "Stop program" })).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Run program" })).toBeNull();
    });

    it("asks the server to stop when pressed", async () => {
        mockReadyState = OPEN;
        renderBar();

        await userEvent.click(screen.getByRole("button", { name: "Stop program" }));

        const sent = mockSendMessage.mock.calls.map((call) => JSON.parse(call[0]));
        expect(sent.some((msg) => msg.kind == STOP_KIND)).toBe(true);
    });
});
