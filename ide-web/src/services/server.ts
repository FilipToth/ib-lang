import axios from "axios";
import { auth } from "./firebase";

export interface IBDiagnostic {
    message: string;
    offset_start: number;
    offset_end: number;
}

export interface IBFile {
    filename: string;
    contents: string;
}

export const runCode = async (code: string): Promise<string> => {
    const headers = await getHeaders();
    const req = await axios.post("http://127.0.0.1:8080/execute", code, {
        headers: headers,
    });

    const data = req.data;
    const output = data.output;
    return output;
};

export const runDiagnostics = async (
    code: string,
    file: string
): Promise<IBDiagnostic[]> => {
    const headers = await getHeaders();
    const params = {
        file: file,
    };

    const req = await axios.post("http://127.0.0.1:8080/diagnostics", code, {
        params: params,
        headers: headers,
    });

    const data = req.data;
    return data;
};

export const getFiles = async (): Promise<IBFile[]> => {
    const headers = await getHeaders();
    const req = await axios.get("http://127.0.0.1:8080/files", {
        headers: headers,
    });

    const data = req.data;
    return data;
};

const getHeaders = async () => {
    const jwt = await auth.currentUser?.getIdToken();
    const headers = {
        Authorization: `Bearer ${jwt}`,
    };

    return headers;
};
