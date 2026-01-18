import { AutoSaver, SaveFn } from "./autosave";

/// Lets the promise callbacks queued so far run.
async function settle() {
    for (let i = 0; i < 10; i++) await Promise.resolve();
}

/// Moves the clock on by `ms`, letting saves land along the way.
async function wait(ms: number) {
    jest.advanceTimersByTime(ms);
    await settle();
}

/// A save the test answers by hand, one request at a time.
function manualServer() {
    const requests: {
        id: string;
        contents: string;
        seq: number;
        signal: AbortSignal;
        resolve: () => void;
        reject: (e: unknown) => void;
    }[] = [];

    const save: SaveFn = (id, contents, seq, signal) =>
        new Promise((resolve, reject) => {
            requests.push({ id, contents, seq, signal, resolve, reject });
        });

    return { save, requests };
}

function setup(save: SaveFn) {
    const onSaved = jest.fn();
    const onError = jest.fn();

    const saver = new AutoSaver({
        save,
        onSaved,
        onError,
        delay: 1000,
        maxWait: 5000,
        timeout: 10000,
        retryDelay: 1000,
        maxRetryDelay: 8000,
    });

    return { saver, onSaved, onError };
}

describe("AutoSaver", () => {
    beforeEach(() => jest.useFakeTimers());
    afterEach(() => jest.useRealTimers());

    it("saves the latest contents once edits pause", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver, onSaved } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(500);
        saver.changed("a", "xy");
        await wait(999);
        expect(save).not.toHaveBeenCalled();

        await wait(1);
        expect(save).toHaveBeenCalledTimes(1);
        expect(save.mock.calls[0].slice(0, 2)).toEqual(["a", "xy"]);
        expect(onSaved).toHaveBeenCalledWith("a", "xy");
        expect(saver.isDirty("a")).toBe(false);
    });

    /// Someone typing without a break still has their work saved.
    it("saves during a steady stream of edits by maxWait", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver } = setup(save);
        saver.known("a", "");

        for (let i = 1; i <= 10; i++) {
            saver.changed("a", "x".repeat(i));
            await wait(500);
        }

        expect(save).toHaveBeenCalled();
    });

    it("does not save contents edited back to what is stored", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver } = setup(save);
        saver.known("a", "same");

        saver.changed("a", "other");
        saver.changed("a", "same");
        await wait(10000);

        expect(save).not.toHaveBeenCalled();
        expect(saver.isDirty("a")).toBe(false);
    });

    /// The hang this was written for: a request that never answers used to
    /// leave the file marked as saving forever.
    it("gives up on a hung save and retries it", async () => {
        const server = manualServer();
        const { saver, onSaved, onError } = setup(server.save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);
        expect(server.requests).toHaveLength(1);

        // never answered
        await wait(10000);
        expect(server.requests[0].signal.aborted).toBe(true);
        expect(onError).toHaveBeenCalledTimes(1);

        await wait(1000);
        expect(server.requests).toHaveLength(2);

        server.requests[1].resolve();
        await settle();
        expect(onSaved).toHaveBeenCalledWith("a", "x");
        expect(saver.isDirty("a")).toBe(false);
    });

    /// The other half of it: a failed save used to wait for another edit.
    it("retries a failed save with no further edits, backing off", async () => {
        const save = jest
            .fn()
            .mockRejectedValueOnce(new Error("offline"))
            .mockRejectedValueOnce(new Error("offline"))
            .mockResolvedValue(undefined);
        const { saver, onSaved, onError } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);
        expect(save).toHaveBeenCalledTimes(1);

        // first retry after 1s
        await wait(1000);
        expect(save).toHaveBeenCalledTimes(2);

        // the second after 2s
        await wait(1999);
        expect(save).toHaveBeenCalledTimes(2);
        await wait(1);
        expect(save).toHaveBeenCalledTimes(3);

        expect(onError.mock.calls.map((c) => c[2])).toEqual([1, 2]);
        expect(onSaved).toHaveBeenCalledWith("a", "x");
    });

    it("caps the wait between retries", async () => {
        const save = jest.fn().mockRejectedValue(new Error("down"));
        const { saver } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        // 1s delay, then retries after 1, 2, 4, 8, 8, 8 seconds
        const total = 1000 + 1000 + 2000 + 4000 + 8000 + 8000 + 8000;
        for (let t = 0; t < total; t += 1000) await wait(1000);

        expect(save).toHaveBeenCalledTimes(7);
    });

    it("sends edits made during a save once it lands", async () => {
        const server = manualServer();
        const { saver, onSaved } = setup(server.save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);

        saver.changed("a", "xy");
        // the pause passes while the first save is still out
        await wait(1000);
        expect(server.requests).toHaveLength(1);

        server.requests[0].resolve();
        await settle();
        expect(saver.isDirty("a")).toBe(true);
        expect(server.requests).toHaveLength(2);
        expect(server.requests[1].contents).toBe("xy");

        server.requests[1].resolve();
        await settle();
        expect(onSaved).toHaveBeenLastCalledWith("a", "xy");
        expect(saver.isDirty("a")).toBe(false);
    });

    it("keeps one save of a file in flight at a time", async () => {
        const server = manualServer();
        const { saver } = setup(server.save);
        saver.known("a", "");

        for (let i = 1; i <= 5; i++) {
            saver.changed("a", "x".repeat(i));
            await wait(1000);
        }

        expect(server.requests).toHaveLength(1);
    });

    it("numbers saves in increasing order", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);
        saver.changed("a", "xy");
        await wait(1000);

        const [first, second] = save.mock.calls.map((c) => c[2]);
        expect(second).toBeGreaterThan(first);
    });

    it("saves every file on its own", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver } = setup(save);
        saver.known("a", "");
        saver.known("b", "");

        saver.changed("a", "1");
        saver.changed("b", "2");
        await wait(1000);

        const saved = save.mock.calls.map((c) => [c[0], c[1]]);
        expect(saved).toEqual([
            ["a", "1"],
            ["b", "2"],
        ]);
    });

    it("saves at once on flush", async () => {
        const save = jest.fn().mockResolvedValue(undefined);
        const { saver } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        saver.flush();
        await settle();

        expect(save).toHaveBeenCalledTimes(1);

        // the pending pause does not send it again
        await wait(1000);
        expect(save).toHaveBeenCalledTimes(1);
    });

    it("stops saving a forgotten file", async () => {
        const save = jest.fn().mockRejectedValue(new Error("gone"));
        const { saver } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);
        saver.forget("a");
        await wait(60000);

        expect(save).toHaveBeenCalledTimes(1);
    });

    it("survives a save that throws instead of rejecting", async () => {
        const save = jest
            .fn()
            .mockImplementationOnce(() => {
                throw new Error("sync");
            })
            .mockResolvedValue(undefined);
        const { saver, onSaved } = setup(save);
        saver.known("a", "");

        saver.changed("a", "x");
        await wait(1000);
        await wait(1000);

        expect(onSaved).toHaveBeenCalledWith("a", "x");
    });
});
