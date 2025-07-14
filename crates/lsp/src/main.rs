use dashmap::DashMap;
use rigz_ast::format;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter_rigz::{HIGHLIGHTS_QUERY, INJECTIONS_QUERY, LANGUAGE, NAMES};

#[derive(Debug)]
struct Backend {
    client: Client,
    files: DashMap<Url, Rope>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: None,
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.files
            .insert(params.text_document.uri, params.text_document.text.into());
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.files.insert(
            params.text_document.uri,
            Rope::from_str(&params.content_changes[0].text),
        );
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let mut contents = match self.files.get_mut(&params.text_document.uri) {
            None => return Ok(None),
            Some(s) => s,
        };
        let rope = contents.value_mut();
        let s = rope.to_string();
        let l = s.len();
        let f = format(s);
        let start = offset_to_position(0, &rope);
        let end = offset_to_position(l, &rope);
        *rope = Rope::from_str(&f);
        let update = match start.map(|s| end.map(|e| Range::new(s, e)).map(|r| TextEdit::new(r, f)))
        {
            Some(Some(t)) => vec![t],
            _ => vec![],
        };
        Ok(Some(update))
    }
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            files: Default::default(),
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
