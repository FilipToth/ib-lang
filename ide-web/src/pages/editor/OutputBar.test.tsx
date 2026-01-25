// the socket and firebase are stood in for: this is about what the button does,
// not about reaching a server
const mockSendMessage = jest.fn();
let mockReadyState = 1;
/// Records what the component asked the socket for, so a test can check the
/// url and the subprotocols it connects with.
const mockUseWebSocket = jest.fn();
/// The frame the component sees on mount. Stands in for the server, which the
/// socket would otherwise have to be reached to hear from.
let mockLastMessage: { data: string } | null = null;

jest.mock("react-use-websocket", () => ({
    __esModule: true,
    default: (...args: unknown[]) => {
        mockUseWebSocket(...args);
        return {
            sendMessage: mockSendMessage,
            lastMessage: mockLastMessage,
            readyState: mockReadyState,
        };
    },
    ReadyState: {
        UNINSTANTIATED: -1,
        CONNECTING: 0,
        OPEN: 1,
        CLOSING: 2,
        CLOSED: 3,
    },
}));

/// null stands for signed out. Read through a getter so a test can change it
/// after the module is loaded.
let mockToken: string | null = null;

jest.mock("services/firebase", () => ({
    auth: {
        get currentUser() {
            return mockToken == null
                ? null
                : { getIdToken: async () => mockToken };
        },
    },
}));

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import OutputBar from "./OutputBar";

const OPEN = 1;
const CLOSED = 3;

/// The kinds the server and client agree on, by position.
const STOP_KIND = 5;
const USAGE_KIND = 6;

/// Must match AUTH_SUBPROTOCOL in OutputBar.tsx and ws.rs.
const AUTH_SUBPROTOCOL = "ib-auth-v1";

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
        mockUseWebSocket.mockReset();
        mockReadyState = CLOSED;
        mockToken = null;
        mockLastMessage = null;
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

/// The token authenticates the socket, and a url would carry it into access
/// logs and browser history. It travels as a subprotocol instead, which the
/// server reads back out of the handshake.
describe("the socket's credentials", () => {
    beforeEach(() => {
        mockSendMessage.mockReset();
        mockUseWebSocket.mockReset();
        mockReadyState = CLOSED;
        mockToken = null;
        mockLastMessage = null;
    });

    it("offers the token as a subprotocol, and keeps it out of the url", async () => {
        mockToken = "header.payload.signature";
        renderBar();

        await userEvent.click(screen.getByRole("button", { name: "Run program" }));

        // the token is fetched before the socket is asked for, so the render
        // that carries it lands a tick after the click
        await waitFor(() => {
            const [url] = mockUseWebSocket.mock.calls.at(-1) as [string | null];
            expect(url).not.toBeNull();
        });

        const [url, options] = mockUseWebSocket.mock.calls.at(-1) as [
            string | null,
            { protocols?: string[] },
        ];

        expect(url).toBe(process.env.REACT_APP_WEBSOCKETS_URL);
        expect(url).not.toContain("token=");
        expect(options.protocols).toEqual([
            AUTH_SUBPROTOCOL,
            "header.payload.signature",
        ]);
    });

    it("does not connect at all when signed out", async () => {
        renderBar();

        await userEvent.click(screen.getByRole("button", { name: "Run program" }));

        const connectedWith = mockUseWebSocket.mock.calls.map((call) => call[0]);
        expect(connectedWith.every((url) => url == null)).toBe(true);
    });
});

/// A run's cost is reported however it ended, so a program stopped for going
/// past a budget still says how far it got.
describe("what a run cost", () => {
    beforeEach(() => {
        mockSendMessage.mockReset();
        mockUseWebSocket.mockReset();
        mockReadyState = CLOSED;
        mockToken = null;
        mockLastMessage = null;
    });

    it("shows the spend the server reports", async () => {
        mockLastMessage = {
            data: JSON.stringify({
                kind: USAGE_KIND,
                payload: "",
                steps: 1234,
                elements: 20,
            }),
        };

        renderBar();

        expect(
            await screen.findByText("1,234 steps, 20 items stored"),
        ).toBeInTheDocument();
    });

    it("shows nothing before a run has reported one", () => {
        renderBar();
        expect(screen.queryByText(/items stored/)).toBeNull();
    });
});
