/// Sends one file's contents to the server. `seq` orders the saves, and the
/// request is abandoned when `signal` aborts.
export type SaveFn = (
    id: string,
    contents: string,
    seq: number,
    signal: AbortSignal,
) => Promise<void>;

export interface AutoSaverOptions {
    save: SaveFn;
    /// The server has stored `contents` as file `id`.
    onSaved: (id: string, contents: string) => void;
    /// A save of file `id` failed; `failures` counts them since the last one
    /// that succeeded. It is retried regardless.
    onError: (id: string, error: unknown, failures: number) => void;
    /// How long edits have to pause before a save.
    delay?: number;
    /// The longest edits can go unsaved while they keep coming.
    maxWait?: number;
    /// How long a save may take before it is given up on and retried.
    timeout?: number;
    /// The first retry's wait. Each failure doubles it, up to `maxRetryDelay`.
    retryDelay?: number;
    maxRetryDelay?: number;
}

interface FileState {
    /// The contents as last edited.
    latest: string;
    /// The contents the server last confirmed storing, if known.
    saved: string | undefined;
    inFlight: boolean;
    timer: ReturnType<typeof setTimeout> | null;
    /// When the oldest unsaved edit was made, for `maxWait`.
    dirtySince: number | null;
    failures: number;
}

/// Keeps files saved as they are edited.
///
/// Each file is saved once edits to it pause, at most one save of it is in
/// flight at a time, and the newest contents always follow once it lands. A
/// save that fails or hangs past `timeout` is retried, backing off, until one
/// goes through; it never waits on another edit to try again.
export class AutoSaver {
    private files = new Map<string, FileState>();
    private lastSeq = 0;

    private save: SaveFn;
    private onSaved: AutoSaverOptions["onSaved"];
    private onError: AutoSaverOptions["onError"];
    private delay: number;
    private maxWait: number;
    private timeout: number;
    private retryDelay: number;
    private maxRetryDelay: number;

    constructor(options: AutoSaverOptions) {
        this.save = options.save;
        this.onSaved = options.onSaved;
        this.onError = options.onError;
        this.delay = options.delay ?? 1000;
        this.maxWait = options.maxWait ?? 5000;
        this.timeout = options.timeout ?? 15000;
        this.retryDelay = options.retryDelay ?? 1000;
        this.maxRetryDelay = options.maxRetryDelay ?? 30000;
    }

    /// Records that the server already has `contents` for file `id`, as when
    /// the files are loaded or one is created.
    known(id: string, contents: string) {
        const file = this.state(id, contents);
        file.saved = contents;
    }

    /// Records an edit of file `id`, to be saved once edits pause.
    changed(id: string, contents: string) {
        const file = this.state(id, contents);
        file.latest = contents;

        if (!this.dirty(file)) {
            // edited back to what is stored; nothing left to send
            this.clearTimer(file);
            file.dirtySince = null;
            return;
        }

        const now = Date.now();
        file.dirtySince ??= now;

        // a steady stream of edits never pauses, so it is cut off at maxWait
        const wait = Math.min(
            this.delay,
            Math.max(0, file.dirtySince + this.maxWait - now),
        );

        this.schedule(id, file, wait);
    }

    /// Saves every file with unsaved edits now, without waiting for a pause.
    flush() {
        this.files.forEach((file, id) => {
            if (!this.dirty(file)) return;

            this.clearTimer(file);
            this.send(id, file);
        });
    }

    /// Stops saving file `id`, as when it is deleted.
    forget(id: string) {
        const file = this.files.get(id);
        if (file != null) this.clearTimer(file);

        this.files.delete(id);
    }

    /// Whether file `id` has edits the server has not confirmed.
    isDirty(id: string) {
        const file = this.files.get(id);
        return file != null && this.dirty(file);
    }

    private state(id: string, contents: string): FileState {
        let file = this.files.get(id);

        if (file == null) {
            file = {
                latest: contents,
                saved: undefined,
                inFlight: false,
                timer: null,
                dirtySince: null,
                failures: 0,
            };
            this.files.set(id, file);
        }

        return file;
    }

    private dirty(file: FileState) {
        return file.latest !== file.saved;
    }

    private clearTimer(file: FileState) {
        if (file.timer != null) clearTimeout(file.timer);
        file.timer = null;
    }

    private schedule(id: string, file: FileState, wait: number) {
        this.clearTimer(file);
        file.timer = setTimeout(() => {
            file.timer = null;
            this.send(id, file);
        }, wait);
    }

    /// Numbers increase across reloads too, since they come from the clock,
    /// and stay whole numbers well within what a double holds exactly.
    private nextSeq() {
        this.lastSeq = Math.max(Date.now() * 1000, this.lastSeq + 1);
        return this.lastSeq;
    }

    private send(id: string, file: FileState) {
        // the save in flight picks up these edits once it lands
        if (file.inFlight || !this.dirty(file)) return;

        const contents = file.latest;
        const seq = this.nextSeq();
        const abort = new AbortController();

        file.inFlight = true;
        file.dirtySince = null;

        let timer: ReturnType<typeof setTimeout> | null = null;
        const timedOut = new Promise<never>((_, reject) => {
            timer = setTimeout(() => {
                abort.abort();
                reject(new Error("The save timed out"));
            }, this.timeout);
        });

        let attempt: Promise<void>;
        try {
            attempt = this.save(id, contents, seq, abort.signal);
        } catch (error) {
            attempt = Promise.reject(error);
        }

        Promise.race([attempt, timedOut])
            .then(
                () => {
                    file.saved = contents;
                    file.failures = 0;
                    this.onSaved(id, contents);
                },
                (error) => {
                    file.failures += 1;
                    this.onError(id, error, file.failures);
                },
            )
            .catch((error) => {
                // a throwing callback must not stall the file
                console.error("autosave callback failed", error);
            })
            .finally(() => {
                if (timer != null) clearTimeout(timer);
                file.inFlight = false;

                // deleted while the save was out
                if (this.files.get(id) !== file) return;
                if (!this.dirty(file)) return;

                if (file.failures > 0) {
                    const backoff = Math.min(
                        this.retryDelay * 2 ** (file.failures - 1),
                        this.maxRetryDelay,
                    );
                    this.schedule(id, file, backoff);
                } else if (file.timer == null) {
                    // edited while the save was out, and nothing is waiting
                    // to send it
                    this.send(id, file);
                }
            });
    }
}
