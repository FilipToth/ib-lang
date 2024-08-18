use std::fs::OpenOptions;
use std::io::Write;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {

        let mut file = OpenOptions::new()
            .append(true)
            .open("lsp.log")
            .expect("err opening file");

        file.write("Received initialize\n".as_bytes()).expect("err writing to file");
        file.flush();

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "IB-LSP".to_string(),
                version: Some("0.01".to_string())
            }),
            capabilities: ServerCapabilities {
                inlay_hint_provider: None,
                text_document_sync: None,
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None
                }),
                execute_command_provider: None,
                workspace: None,
                semantic_tokens_provider: None,
                definition_provider: None,
                references_provider: None,
                rename_provider: None,
                ..ServerCapabilities::default()
            }
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = || -> Option<Vec<CompletionItem>> {
            let mut vec: Vec<CompletionItem> = Vec::new();
            let item = CompletionItem {
                label: "sample_completion".to_string(),
                insert_text: Some("sample_completion".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("sample_completion".to_string()),
                ..Default::default()
            };

            vec.push(item);
            Some(vec)
        }();

        Ok(completions.map(CompletionResponse::Array))
    }
}

#[tokio::main]
async fn main() {

    let mut file = OpenOptions::new()
            .append(true)
            .open("lsp.log")
            .expect("err opening file");

    file.write("initializing server\n".as_bytes()).expect("err writing to file");
    file.flush();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
