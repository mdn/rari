use std::path::PathBuf;
use std::sync::Arc;

use rari_doc::find::doc_pages_from_slugish;
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::doc_from_raw;
use rari_tools::fix::issues::get_fixable_issues;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DidSaveTextDocumentParams, Position, Range,
};
use tree_sitter_md::MarkdownParser;

fn text_doc_change_to_tree_sitter_edit(
    change: &tower_lsp::lsp_types::TextDocumentContentChangeEvent,
    doc: &lsp_textdocument::FullTextDocument,
) -> Result<tree_sitter::InputEdit, &'static str> {
    let range = change.range.as_ref().ok_or("Invalid edit range")?;
    let start = range.start;
    let end = range.end;

    let start_byte = doc.offset_at(start) as usize;
    let old_end_byte = doc.offset_at(end) as usize;
    let new_end_byte = start_byte + change.text.len();

    let new_end_pos = doc.position_at(new_end_byte as u32);

    Ok(tree_sitter::InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position: tree_sitter::Point {
            row: start.line as usize,
            column: start.character as usize,
        },
        old_end_position: tree_sitter::Point {
            row: end.line as usize,
            column: end.character as usize,
        },
        new_end_position: tree_sitter::Point {
            row: new_end_pos.line as usize,
            column: new_end_pos.character as usize,
        },
    })
}

pub(crate) struct Backend {
    client: tower_lsp::Client,
    curr_doc_path: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
    parser: std::sync::Arc<tokio::sync::Mutex<tree_sitter::Parser>>,
    md_parser: std::sync::Arc<tokio::sync::Mutex<tree_sitter_md::MarkdownParser>>,
    curr_doc: std::sync::Arc<tokio::sync::Mutex<Option<lsp_textdocument::FullTextDocument>>>,
    tree: std::sync::Arc<tokio::sync::Mutex<Option<tree_sitter::Tree>>>,
    md_tree: std::sync::Arc<tokio::sync::Mutex<Option<tree_sitter_md::MarkdownTree>>>,
    kw_docs: crate::keywords::KeywordDocsMap,
}

impl Backend {
    pub(crate) fn new(client: tower_lsp::Client) -> Self {
        Self {
            client,
            parser: std::sync::Arc::new(
                tokio::sync::Mutex::new(crate::parser::initialise_parser()),
            ),
            curr_doc_path: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            md_parser: std::sync::Arc::new(tokio::sync::Mutex::new(MarkdownParser::default())),
            curr_doc: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            tree: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            md_tree: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            kw_docs: crate::keywords::load_kw_docs(),
        }
    }
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for Backend {
    async fn initialize(
        &self,
        _: tower_lsp::lsp_types::InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<tower_lsp::lsp_types::InitializeResult> {
        Ok(tower_lsp::lsp_types::InitializeResult {
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: String::from("mdn-lsp"),
                version: Some(String::from("0.0.1")),
            }),
            capabilities: tower_lsp::lsp_types::ServerCapabilities {
                text_document_sync: Some(tower_lsp::lsp_types::TextDocumentSyncCapability::Kind(
                    tower_lsp::lsp_types::TextDocumentSyncKind::INCREMENTAL,
                )),

                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                completion_provider: Some(tower_lsp::lsp_types::CompletionOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    trigger_characters: Some(vec![
                        "/".to_string(),
                        "-".to_string(),
                        ":".to_string(),
                        ".".to_string(),
                    ]),
                    ..Default::default()
                }),
                ..tower_lsp::lsp_types::ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: tower_lsp::lsp_types::InitializedParams) {
        self.client
            .log_message(tower_lsp::lsp_types::MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: tower_lsp::lsp_types::DidOpenTextDocumentParams) {
        let mut curr_doc = self.curr_doc.lock().await;
        let mut curr_doc_path = self.curr_doc_path.lock().await;
        let mut tree = self.tree.lock().await;
        let mut parser = self.parser.lock().await;
        let mut md_tree = self.md_tree.lock().await;
        let mut md_parser = self.md_parser.lock().await;

        *curr_doc = Some(lsp_textdocument::FullTextDocument::new(
            params.text_document.language_id.clone(),
            params.text_document.version,
            params.text_document.text.clone(),
        ));
        *curr_doc_path = params.text_document.uri.to_file_path().ok();
        *tree = parser.parse(&params.text_document.text, None);
        *md_tree = md_parser.parse(params.text_document.text.as_bytes(), None);
    }

    async fn did_change(&self, params: tower_lsp::lsp_types::DidChangeTextDocumentParams) {
        let mut curr_doc = self.curr_doc.lock().await;
        let mut tree = self.tree.lock().await;
        let mut md_tree = self.md_tree.lock().await;

        if let Some(ref mut doc) = *curr_doc {
            doc.update(&params.content_changes, params.text_document.version);
            for change in params.content_changes.iter() {
                match text_doc_change_to_tree_sitter_edit(change, doc) {
                    Ok(edit) => {
                        if let Some(ref mut curr_tree) = *tree {
                            curr_tree.edit(&edit);
                        }
                        if let Some(ref mut curr_md_tree) = *md_tree {
                            curr_md_tree.edit(&edit);
                        }
                    }
                    Err(err) => {
                        self.client
                            .log_message(
                                tower_lsp::lsp_types::MessageType::ERROR,
                                format!("Bad edit info, failed to edit tree: {}", err),
                            )
                            .await;
                    }
                }
            }
        }
    }

    async fn hover(
        &self,
        params: tower_lsp::lsp_types::HoverParams,
    ) -> tower_lsp::jsonrpc::Result<Option<tower_lsp::lsp_types::Hover>> {
        let curr_doc = self.curr_doc.lock().await;
        let mut tree = self.tree.lock().await;
        let mut parser = self.parser.lock().await;

        let doc = match &*curr_doc {
            Some(doc) => doc,
            _ => return Ok(None),
        };

        let keyword = crate::position::retrieve_keyword_at_position(
            doc.get_content(None),
            &mut parser,
            &mut tree,
            params.text_document_position_params.position.line as usize,
            params.text_document_position_params.position.character as usize,
        );

        match keyword {
            Some(keyword) => {
                if let Some(doc_content) = self.kw_docs.get(&keyword.as_str()) {
                    let hover_contents = tower_lsp::lsp_types::HoverContents::Markup(
                        tower_lsp::lsp_types::MarkupContent {
                            kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                            value: doc_content.to_string(),
                        },
                    );
                    let hover = tower_lsp::lsp_types::Hover {
                        contents: hover_contents,
                        range: None,
                    };
                    Ok(Some(hover))
                } else {
                    self.client
                        .log_message(
                            tower_lsp::lsp_types::MessageType::WARNING,
                            format!("Documentation for keyword '{}' not found.", keyword),
                        )
                        .await;
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    async fn completion(
        &self,
        params: tower_lsp::lsp_types::CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<tower_lsp::lsp_types::CompletionResponse>> {
        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                "Checking completion...",
            )
            .await;
        let curr_doc = self.curr_doc.lock().await;

        let doc = match &*curr_doc {
            Some(doc) => doc,
            _ => return Ok(None),
        };

        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                "Checking completion. Got doc ..",
            )
            .await;

        let md = doc.get_content(None);
        let mut md_tree = self.md_tree.lock().await;
        let mut md_parser = self.md_parser.lock().await;

        let element = crate::position::retrieve_element_at_position(
            md,
            &mut md_parser,
            &mut md_tree,
            params.text_document_position.position.line as usize,
            params.text_document_position.position.character as usize,
        );
        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!("Checking completion. Got element {element:?}"),
            )
            .await;
        if let Some(crate::position::Element::Link { link }) = element {
            self.client
                .log_message(
                    tower_lsp::lsp_types::MessageType::INFO,
                    format!("Checking completion for {link}"),
                )
                .await;
            let items = doc_pages_from_slugish(&link, rari_types::locale::Locale::EnUs)
                .unwrap()
                .into_iter()
                .map(|item| tower_lsp::lsp_types::CompletionItem {
                    label: item.url().to_string(),
                    kind: Some(tower_lsp::lsp_types::CompletionItemKind::TEXT),
                    ..tower_lsp::lsp_types::CompletionItem::default()
                })
                .collect();

            return Ok(Some(tower_lsp::lsp_types::CompletionResponse::List(
                tower_lsp::lsp_types::CompletionList {
                    is_incomplete: true,
                    items,
                },
            )));
        }

        let mut tree = self.tree.lock().await;
        let mut parser = self.parser.lock().await;
        if let Some(keyword) = crate::position::retrieve_keyword_at_position(
            doc.get_content(None),
            &mut parser,
            &mut tree,
            params.text_document_position.position.line as usize,
            params.text_document_position.position.character as usize,
        ) {
            let documentation =
                self.kw_docs
                    .get(keyword.as_str())
                    .map(|doc| tower_lsp::lsp_types::MarkupContent {
                        kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                        value: doc.to_string(),
                    });

            let item = tower_lsp::lsp_types::CompletionItem {
                label: "cssxref".to_string(),
                kind: Some(tower_lsp::lsp_types::CompletionItemKind::KEYWORD),
                documentation: documentation
                    .map(tower_lsp::lsp_types::Documentation::MarkupContent),
                ..tower_lsp::lsp_types::CompletionItem::default()
            };
            let items = vec![item];

            return Ok(Some(tower_lsp::lsp_types::CompletionResponse::List(
                tower_lsp::lsp_types::CompletionList {
                    is_incomplete: true,
                    items,
                },
            )));
        }

        Ok(None)
    }
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let curr_doc = self.curr_doc.lock().await;
        let curr_doc_path = self.curr_doc_path.lock().await;

        let (doc, path) = match (&*curr_doc, &*curr_doc_path) {
            (Some(doc), Some(path)) => (doc, path),
            _ => return,
        };

        let page = match doc_from_raw(doc.get_content(None).to_string(), path) {
            Ok(doc) => Page::Doc(Arc::new(doc)),
            Err(e) => {
                self.client
                    .log_message(tower_lsp::lsp_types::MessageType::ERROR, format!("{e}"))
                    .await;
                return;
            }
        };

        let issues = match get_fixable_issues(&page) {
            Ok(issues) => issues,
            Err(e) => {
                self.client
                    .log_message(tower_lsp::lsp_types::MessageType::ERROR, format!("{e}"))
                    .await;
                return;
            }
        };

        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!("Issues: {issues:?}"),
            )
            .await;
        let diagnostics = issues
            .into_iter()
            .map(|issue| {
                let display_issue = issue.display_issue();
                Diagnostic {
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: display_issue.explanation.clone().unwrap_or_default(),
                    range: Range::new(
                        Position::new(
                            display_issue.line.unwrap_or(1).saturating_sub(1) as u32,
                            display_issue.column.unwrap_or(1).saturating_sub(1) as u32,
                        ),
                        Position::new(
                            display_issue.end_line.unwrap_or(1).saturating_sub(1) as u32,
                            display_issue.end_column.unwrap_or(1) as u32,
                        ),
                    ),
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();
        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!("Diagnostics: {diagnostics:?}"),
            )
            .await;
        self.client
            .publish_diagnostics(params.text_document.uri.clone(), diagnostics, None)
            .await;
    }
}
