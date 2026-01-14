// the real module pulls in firebase, which jest cannot transform, and the hook
// only ever touches these two pieces of it
let mockListener: ((user: unknown) => void) | null = null;
const mockUnsubscribe = jest.fn();

jest.mock("./firebase", () => ({ auth: { currentUser: null } }));
jest.mock("firebase/auth", () => ({
    onAuthStateChanged: (_auth: unknown, callback: (user: unknown) => void) => {
        mockListener = callback;
        return mockUnsubscribe;
    },
}));

import { renderHook, act } from "@testing-library/react";
import useAuthUser from "./useAuthUser";

const signedIn = { uid: "abc" };

describe("useAuthUser", () => {
    beforeEach(() => {
        mockListener = null;
        mockUnsubscribe.mockClear();
    });

    /// The reason the hook exists: a reload has a stored session that firebase
    /// has not restored yet, and treating that moment as signed out is what
    /// redirected people to the sign-in page.
    it("waits for firebase before saying who is signed in", () => {
        const { result } = renderHook(() => useAuthUser());

        expect(result.current).toEqual({ user: null, loading: true });

        act(() => mockListener!(signedIn));

        expect(result.current).toEqual({ user: signedIn, loading: false });
    });

    it("reports a signed-out user once firebase has looked", () => {
        const { result } = renderHook(() => useAuthUser());

        act(() => mockListener!(null));

        expect(result.current).toEqual({ user: null, loading: false });
    });

    it("stops listening when it goes away", () => {
        const { unmount } = renderHook(() => useAuthUser());

        unmount();

        expect(mockUnsubscribe).toHaveBeenCalled();
    });
});
