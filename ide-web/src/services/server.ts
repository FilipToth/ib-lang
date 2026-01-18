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

export const runDiagnostics = async (code: string): Promise<IBDiagnostic[]> => {
    const headers = await getHeaders();
    const req = await axios.post(`${API_BASE}diagnostics`, code, {
        headers: headers,
    });

    const data = req.data;
    return data;
};

export interface IBControlFlowGraph {
    /// Graphviz DOT, or null when the program does not compile.
    dot: string | null;
    /// Why it could not be drawn. Empty when `dot` is set.
    diagnostics: IBDiagnostic[];
}

export const getControlFlowGraph = async (
    code: string
): Promise<IBControlFlowGraph> => {
    const headers = await getHeaders();
    const req = await axios.post(`${API_BASE}control-flow`, code, {
        headers: headers,
    });

    return req.data;
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

    const req = await axios.post(`${API_BASE}create`, undefined, {
        params: params,
        headers: headers,
    });

    ensureSuccess(req.data);
};

export const deleteFile = async (id: string) => {
    const headers = await getHeaders();
    const params = {
        id: id,
    };

    const req = await axios.post(`${API_BASE}delete`, undefined, {
        params: params,
        headers: headers,
    });

    ensureSuccess(req.data);
};

/// Stores `contents` as file `id`. The server refuses a save whose `seq` is no
/// higher than one it already wrote, so a late request cannot put back older
/// contents. `signal` abandons the request.
export const saveFile = async (
    id: string,
    contents: string,
    seq: number,
    signal?: AbortSignal
) => {
    const headers = await getHeaders();
    const params = {
        id: id,
        seq: seq,
    };

    const req = await axios.post(`${API_BASE}save`, contents, {
        params: params,
        headers: headers,
        signal: signal,
    });

    ensureSuccess(req.data);
};

/// The file routes answer a refused request with `{ success: false }` rather
/// than an error status, so that is turned into a rejection here.
const ensureSuccess = (data: { success: boolean }) => {
    if (!data.success) {
        throw new Error("The server refused the request");
    }
};

const getHeaders = async () => {
    const jwt = await auth.currentUser?.getIdToken();
    const headers = {
        Authorization: `Bearer ${jwt}`,
    };

    return headers;
};
