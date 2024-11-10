import axios from "axios";

interface IBDiagnostic {
    message: string;
    line: number;
    col: number;
}

export const runCode = async (code: string): Promise<string> => {
    const req = await axios.post("http://127.0.0.1:8080/execute", code);
    const data = req.data;

    const output = data.output;
    return output;
};

export const runDiagnostics = async (code: string): Promise<IBDiagnostic[]> => {
    const req = await axios.post("http://127.0.0.1:8080/diagnostics", code);

    const data = req.data;
    return data;
};
