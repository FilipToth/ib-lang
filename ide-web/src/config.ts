import { UserConfig } from 'monaco-editor-wrapper';

export const HELLO_LANG_ID = 'ib';
export const HELLO_LANG_EXTENSION = 'ib';

const LS_WS_URL = 'ws://localhost:8080/';

export const helloConfig: UserConfig = {
    wrapperConfig: {
        editorAppConfig: {
            $type: 'classic',
            codeResources: {
                main: {
                    text: '',
                    fileExt: HELLO_LANG_EXTENSION,
                },
            },
            useDiffEditor: false,
            languageDef: {
                languageExtensionConfig: {
                    id: HELLO_LANG_ID,
                    extensions: [HELLO_LANG_EXTENSION],
                },
                theme: {
                    name: 'vs-dark',
                    data: {
                        base: 'vs-dark',
                        inherit: true,
                        rules: [],
                        encodedTokensColors: [],
                        colors: { },
                    }
                }
            },
        },
    },
    languageClientConfig: {
        languageId: HELLO_LANG_ID,
        options: {
            $type: 'WebSocketUrl',
            url: LS_WS_URL,
            startOptions: {
                onCall: () => {
                    console.log('Connected to socket.');
                },
                reportStatus: true,
            },
            stopOptions: {
                onCall: () => {
                    console.log('Disconnected from socket.');
                },
                reportStatus: true,
            },
        },
    },
};
