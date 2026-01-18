import { useState } from "react";
import { IBFile } from "services/server";

/// The text of a pane's editor, and a setter for edits made in it.
///
/// The buffer is filled from `file` whenever a different file is put in front
/// of it, and left alone while no file is (a graph tab), so going back to the
/// file keeps what was typed.
///
/// Which file it holds is state rather than a ref on purpose: React renders a
/// component twice in development and keeps the second run, and a ref written
/// during the first run would make the second one skip the refill -- which is
/// exactly the tab that changes but shows the previous file's code.
export const useFileBuffer = (file: IBFile | null) => {
    const [code, setCode] = useState(file?.contents ?? "");
    const [held, setHeld] = useState(file?.id ?? null);

    if (file != null && held != file.id) {
        setHeld(file.id);
        setCode(file.contents);
    }

    return [code, setCode] as const;
};
