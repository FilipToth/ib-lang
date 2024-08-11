import './App.css';
import MonacoEditor, { EditorDidMount } from 'react-monaco-editor';
import * as monaco from '@codingame/monaco-vscode-editor-api'
import { WebSocketMessageReader, WebSocketMessageWriter, toSocket } from 'vscode-ws-jsonrpc/.';
import { MonacoLanguageClient } from 'monaco-languageclient/.';
import { CloseAction, ErrorAction } from 'vscode-languageclient';

const LANGUAGE_ID = 'ib-lang'
const LANGUAGE_EXTENSION = '.ib'
const LSP_SOCKET_ADDRESS = 'ws://localhost:8080'

const registerLanguage = () => monaco.languages.register({
    id: LANGUAGE_ID,
    aliases: [LANGUAGE_ID],
    extensions: [LANGUAGE_EXTENSION]
});

const createModel = () => monaco.editor.createModel(
    '',
    LANGUAGE_ID,
    monaco.Uri.parse(`file://ib-${Math.random()}${LANGUAGE_EXTENSION}`)
);

const createWebsocket = () => {
    return new Promise((resolve, reject) => {
        const ws = new WebSocket(LSP_SOCKET_ADDRESS);
        ws.onopen = () => {
            const socket = toSocket(ws);
            const reader = new WebSocketMessageReader(socket);
            const writer = new WebSocketMessageWriter(socket);

            const client = new MonacoLanguageClient({
                name: `${LANGUAGE_ID} Language Client`,
                clientOptions: {
					documentSelector: [LANGUAGE_ID],
                    errorHandler: {
                        error: () => ({ action: ErrorAction.Continue }),
                        closed: () => ({ action: CloseAction.DoNotRestart })
                    }
                },
                connectionProvider: {
                    get: () => Promise.resolve({reader, writer})
                }
            });

			client.start();
			resolve(client)
        };

		ws.onerror = (err) => {
			reject(err);
		}
    });
};

const editorDidMount: EditorDidMount = (editor) => {
	registerLanguage();

	const model = createModel();
	editor.setModel(model);

	createWebsocket();
	editor.focus();
};

function App() {
    const options = {
        selectOnLineNumbers: true
    };

    return (
        <MonacoEditor
            height='100vh'
            theme='vs-dark'
            language='javascript'
            options={options}
			editorDidMount={editorDidMount}
        />
    );
}

export default App;
