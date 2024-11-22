import axios from "axios";
import { auth } from "./firebase";

export interface IBDiagnostic {
    message: string;
    line: number;
    col: number;
}

export interface IBFile {
    filename: string;
    contents: string;
}

export const runCode = async (code: string): Promise<string> => {
    const req = await axios.post("http://127.0.0.1:8080/execute", code);
    const data = req.data;

    const output = data.output;
    return output;
};

export const runDiagnostics = async (
    code: string,
    file: string
): Promise<IBDiagnostic[]> => {
    const user = auth.currentUser;
    if (user == null) return [];

    const params = {
        file: file,
        uid: user?.uid,
    };

    const req = await axios.post("http://127.0.0.1:8080/diagnostics", code, {
        params: params,
    });

    const data = req.data;
    return data;
};

export const getFiles = async (): Promise<IBFile[]> => {
    const user = auth.currentUser;
    if (user == null) return [];

    const params = {
        uid: user?.uid,
    };

    const req = await axios.get("http://127.0.0.1:8080/files", {
        params: params,
    });

    const data = req.data;
    console.log(data);
    return data;
};
