import axios from "axios";
import { auth } from "./firebase";

export interface IBDiagnostic {
    message: string;
    offset_start: number;
    offset_end: number;
}

export interface IBFile {
    id: string;
    filename: string;
    contents: string;
}

const API_BASE = process.env.REACT_APP_BACKEND_URL;

export const runCode = async (code: string): Promise<string> => {
    const headers = await getHeaders();
    const req = await axios.post(`${API_BASE}execute`, code, {
        headers: headers,
    });

    const data = req.data;
    const output = data.output;
    return output;
};

export const runDiagnostics = async (file: IBFile): Promise<IBDiagnostic[]> => {
    const headers = await getHeaders();
    const params = {
        id: file.id,
    };

    const req = await axios.post(`${API_BASE}diagnostics`, file.contents, {
        params: params,
        headers: headers,
    });

    const data = req.data;
    return data;
};

export const getFiles = async (): Promise<IBFile[]> => {
    const headers = await getHeaders();
    const req = await axios.get(`${API_BASE}files`, {
        headers: headers,
    });

    const data = req.data;
    return data;
};

export const createFile = async (id: string, filename: string) => {
    const headers = await getHeaders();
    const params = {
        id: id,
        filename: filename,
    };

    await axios.post(`${API_BASE}create`, undefined, {
        params: params,
        headers: headers,
    });
};

export const deleteFile = async (id: string) => {
    const headers = await getHeaders();
    const params = {
        id: id,
    };

    await axios.post(`${API_BASE}delete`, undefined, {
        params: params,
        headers: headers,
    });
};

const getHeaders = async () => {
    const jwt = await auth.currentUser?.getIdToken();
    const headers = {
        Authorization: `Bearer ${jwt}`,
    };

    return headers;
};
