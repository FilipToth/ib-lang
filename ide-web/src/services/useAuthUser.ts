import { User, onAuthStateChanged } from "firebase/auth";
import { useEffect, useState } from "react";
import { auth } from "./firebase";

export interface AuthState {
    user: User | null;
    /// True until firebase has restored, or ruled out, a stored session.
    loading: boolean;
}

/// Subscribes to the signed-in user.
///
/// Firebase restores a persisted session asynchronously, so `auth.currentUser`
/// is still null for the first moments after a reload even when the user is
/// signed in. Reading it directly is what used to sign people out on refresh;
/// this waits for the first callback instead, and re-renders on every later
/// change, so signing out redirects on its own.
const useAuthUser = (): AuthState => {
    const [state, setState] = useState<AuthState>({
        // already known when moving between pages within a session; only a
        // fresh load has to wait
        user: auth.currentUser,
        loading: auth.currentUser == null,
    });

    useEffect(() => {
        return onAuthStateChanged(auth, (user) => {
            setState({ user: user, loading: false });
        });
    }, []);

    return state;
};

export default useAuthUser;
