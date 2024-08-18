## Lang Server

 - https://crates.io/crates/lsp-server/0.7.6

## Web Connection

 - https://nipunamarcus.medium.com/part-2-connect-monaco-react-web-editor-with-language-server-using-websocket-how-hard-can-it-be-aa66d93327a6
 - full web client from the prev post: https://github.com/NipunaMarcus/web-editor/tree/websocket-ls?source=post_page-----aa66d93327a6--------------------------------
 - use `monaco-vscode-editor-api` instead of `monaco-editor`, same apis
 - https://github.dev/TypeFox/monaco-languageclient-ng-example

 look at theia:
  - https://www.npmjs.com/package/theia-core (usess monaco-languageclient)

## Code Analysis
 - use the `?` operator for
 ```rust
 match token {
    Some(t) => t,
    None => return None
 }
 ```
 Could be simplified into `token?`