import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import FileNameDialog from "./FileNameDialog";

function setup(options: {
    initialStem?: string;
    takenNames?: string[];
    dialogOK?: (name: string) => Promise<void>;
}) {
    const dialogOK = jest.fn(options.dialogOK ?? (() => Promise.resolve()));

    render(
        <FileNameDialog
            isOpen
            title="Rename File"
            confirmLabel="Rename"
            initialStem={options.initialStem}
            takenNames={options.takenNames ?? []}
            dialogOK={dialogOK}
            close={() => {}}
        />
    );

    const input = screen.getByLabelText("File name") as HTMLInputElement;
    const type = (value: string) =>
        fireEvent.change(input, { target: { value } });
    const submit = () => fireEvent.submit(input.closest("form")!);

    return { input, type, submit, dialogOK };
}

describe("FileNameDialog", () => {
    it("starts from the current name, without its extension", () => {
        const { input } = setup({ initialStem: "main" });
        expect(input.value).toBe("main");
    });

    /// Submitting the form is what Enter does.
    it("adds the extension and trims the name", async () => {
        const { type, submit, dialogOK } = setup({});

        type("  solver ");
        submit();

        await waitFor(() => expect(dialogOK).toHaveBeenCalledWith("solver.ib"));
    });

    it("refuses a name another file has", async () => {
        const { type, submit, dialogOK } = setup({ takenNames: ["taken.ib"] });

        type("taken");
        submit();

        expect(
            await screen.findByText("A file named taken.ib already exists.")
        ).toBeInTheDocument();
        expect(dialogOK).not.toHaveBeenCalled();
    });

    it("refuses periods and slashes", async () => {
        const { type, submit, dialogOK } = setup({});

        type("../x");
        submit();

        expect(
            await screen.findByText(
                "File names cannot contain periods or slashes."
            )
        ).toBeInTheDocument();
        expect(dialogOK).not.toHaveBeenCalled();
    });

    /// A reason from the server is shown where the name is typed, so another
    /// can be tried without opening the dialog again.
    it("shows why the server refused, until the name changes", async () => {
        const { type, submit } = setup({
            dialogOK: () => Promise.reject(new Error("Server said no.")),
        });

        type("fine");
        submit();
        expect(await screen.findByText("Server said no.")).toBeInTheDocument();

        type("other");
        expect(screen.queryByText("Server said no.")).not.toBeInTheDocument();
    });
});
