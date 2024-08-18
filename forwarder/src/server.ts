import { WebSocketServer } from 'ws';
import { Server } from 'node:http';
import express from 'express';
import { URL } from 'node:url';
import { IWebSocket, WebSocketMessageReader, WebSocketMessageWriter } from 'vscode-ws-jsonrpc'
import { createConnection, createServerProcess, forward } from 'vscode-ws-jsonrpc/server';
import { InitializeParams, InitializeRequest, Message } from 'vscode-languageserver';

const runConfig: LanguageServerRunConfig = {
    serverName: 'ib-lang',
    pathName: '/ib-analyzer',
    serverPort: 8080,
    runCommand: './ib-analyzer',
    runCommandArgs: [
        '', // should be process run path
    ]
};

interface LanguageServerRunConfig {
    serverName: string;
    pathName: string;
    serverPort: number;
    runCommand: string;
    runCommandArgs: string[];
}

const launchLangServer = (socket: IWebSocket) => {
    const reader = new WebSocketMessageReader(socket);
    const writer = new WebSocketMessageWriter(socket);

    const socketConnection = createConnection(reader, writer, () => socket.dispose());
    const serverConnection = createServerProcess(runConfig.serverName, runConfig.runCommand, runConfig.runCommandArgs, undefined);

    if (!serverConnection)
        return;

    forward(socketConnection, serverConnection, (message) => {
        if (Message.isRequest(message)) {
            const msg = JSON.stringify(message);
            console.log(`server received:\n${msg}\n\n`)

            if (message.method !== InitializeRequest.type.method)
                return message;

            const initParams = message.params as InitializeParams;
            initParams.processId = process.pid;
        }

        if (Message.isResponse(message)) {
            const msg = JSON.stringify(message);
            console.log(`server sent:\n${msg}\n\n`)
        }

        return message;
    });
};

const runServer = () => {
    const app = express();
    const server = app.listen(runConfig.serverPort)

    const wss = new WebSocketServer({
        noServer: true,
        perMessageDeflate: false
    });

    server.on('upgrade', (req, socket, head) => {
        const baseUrl = `http://${req.headers.host}/`
        const pathName = req.url !== undefined ? new URL(req.url, baseUrl).pathname : undefined;

        if (pathName !== '/') {
            return;
        }

        wss.handleUpgrade(req, socket, head, (webSocket) => {
            const send = (content: string) => {
                webSocket.send(content, (err) => {
                    if (!err) {
                        return;
                    }

                    throw err;
                });
            };

            const onMessage = (cb: (data: any) => void) => {
                webSocket.on('message', (data) => {
                    cb(data)
                })
            };

            const socket: IWebSocket = {
                send: send,
                onMessage: onMessage,
                onError: (cb) => webSocket.on('error', cb),
                onClose: (cb) => webSocket.on('close', cb),
                dispose: () => webSocket.close()
            };

            if (webSocket.readyState == webSocket.OPEN) {
                launchLangServer(socket);
            } else {
                webSocket.on('open', () => {
                    launchLangServer(socket);
                });
            }

            undefined
        })
    });
};

export { runServer };