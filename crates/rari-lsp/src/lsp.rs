use std::sync::Arc;

use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use lsp_textdocument::FullTextDocument;
use rari_doc::find::doc_pages_from_slugish;
use rari_doc::issues::{DIssue, DisplayIssue};
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::doc_from_raw;
use rari_doc::templ::templs::TEMPL_MAP;
use rari_tools::fix::issues::get_fixable_issues;
use tower_lsp_server::lsp_types::{
    CodeAction, CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability,
    CodeActionResponse, CompletionItem, CompletionItemKind, CompletionList, CompletionOptions,
    CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentChanges, Documentation, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, MarkupContent, MarkupKind, MessageType,
    OneOf, OptionalVersionedTextDocumentIdentifier, Position, Range, ServerCapabilities,
    ServerInfo, TextDocumentContentChangeEvent, TextDocumentEdit, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Uri, WorkspaceEdit,
};
use tower_lsp_server::{jsonrpc, LanguageServer, UriExt};
use tree_sitter::Tree;
use tree_sitter_md::{MarkdownParser, MarkdownTree};

fn text_doc_change_to_tree_sitter_edit(
    change: &TextDocumentContentChangeEvent,
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

pub(crate) struct Document {
    pub full: FullTextDocument,
    pub tree: Option<Tree>,
    pub md_tree: Option<MarkdownTree>,
}

pub(crate) struct Documents {
    documents: DashMap<Uri, Document>,
}

impl Documents {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    pub fn open(&self, uri: Uri, doc: Document) {
        self.documents.insert(uri, doc);
    }

    pub fn get_mut(&self, uri: &Uri) -> Option<RefMut<Uri, Document>> {
        self.documents.get_mut(uri)
    }

    pub fn get(&self, uri: &Uri) -> Option<Ref<Uri, Document>> {
        self.documents.get(uri)
    }
}

pub(crate) struct Backend {
    client: tower_lsp_server::Client,
    docs: Documents,
    parser: std::sync::Arc<tokio::sync::Mutex<tree_sitter::Parser>>,
    md_parser: std::sync::Arc<tokio::sync::Mutex<tree_sitter_md::MarkdownParser>>,
    kw_docs: crate::keywords::KeywordDocsMap,
}

impl Backend {
    pub(crate) fn new(client: tower_lsp_server::Client) -> Self {
        Self {
            client,
            docs: Documents::new(),
            parser: std::sync::Arc::new(
                tokio::sync::Mutex::new(crate::parser::initialise_parser()),
            ),
            md_parser: std::sync::Arc::new(tokio::sync::Mutex::new(MarkdownParser::default())),
            kw_docs: crate::keywords::load_kw_docs(),
        }
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: String::from("mdn-lsp"),
                version: Some(String::from("0.0.1")),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
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
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut parser = self.parser.lock().await;
        let mut md_parser = self.md_parser.lock().await;

        let doc = Document {
            full: lsp_textdocument::FullTextDocument::new(
                params.text_document.language_id.clone(),
                params.text_document.version,
                params.text_document.text.clone(),
            ),
            tree: parser.parse(&params.text_document.text, None),
            md_tree: md_parser.parse(params.text_document.text.as_bytes(), None),
        };

        self.docs.open(params.text_document.uri, doc)
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut curr_doc = self.docs.get_mut(&params.text_document.uri);

        if let Some(ref mut doc) = curr_doc {
            doc.full
                .update(&params.content_changes, params.text_document.version);
            for change in params.content_changes.iter() {
                match text_doc_change_to_tree_sitter_edit(change, &doc.full) {
                    Ok(edit) => {
                        if let Some(ref mut curr_tree) = doc.tree {
                            curr_tree.edit(&edit);
                        }
                        if let Some(ref mut curr_md_tree) = doc.md_tree {
                            curr_md_tree.edit(&edit);
                        }
                    }
                    Err(err) => {
                        self.client
                            .log_message(
                                MessageType::ERROR,
                                format!("Bad edit info, failed to edit tree: {}", err),
                            )
                            .await;
                    }
                }
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let mut curr_doc = self
            .docs
            .get_mut(&params.text_document_position_params.text_document.uri);
        let mut parser = self.parser.lock().await;

        if let Some(ref mut doc) = curr_doc {
            let keyword = crate::position::retrieve_keyword_at_position(
                doc,
                &mut parser,
                params.text_document_position_params.position.line as usize,
                params.text_document_position_params.position.character as usize,
            );

            match keyword {
                Some(keyword) => {
                    if let Some(t) = self.kw_docs.get(
                        keyword
                            .as_str()
                            .to_ascii_lowercase()
                            .replace("-", "_")
                            .as_str(),
                    ) {
                        let hover_contents = HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: t.outline.to_string(),
                        });
                        let hover = Hover {
                            contents: hover_contents,
                            range: None,
                        };
                        Ok(Some(hover))
                    } else {
                        self.client
                            .log_message(
                                MessageType::WARNING,
                                format!("Documentation for keyword '{}' not found.", keyword),
                            )
                            .await;
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, "Checking completion...")
            .await;
        let mut curr_doc = self
            .docs
            .get_mut(&params.text_document_position.text_document.uri);

        if let Some(ref mut doc) = curr_doc {
            self.client
                .log_message(MessageType::INFO, "Checking completion. Got doc ..")
                .await;

            let mut md_parser = self.md_parser.lock().await;

            let element = crate::position::retrieve_element_at_position(
                doc,
                &mut md_parser,
                params.text_document_position.position.line as usize,
                params.text_document_position.position.character as usize,
            );
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Checking completion. Got element {element:?}"),
                )
                .await;
            if let Some(crate::position::Element::Link { link }) = element {
                self.client
                    .log_message(MessageType::INFO, format!("Checking completion for {link}"))
                    .await;
                let items = doc_pages_from_slugish(&link, rari_types::locale::Locale::EnUs)
                    .unwrap()
                    .into_iter()
                    .map(|item| CompletionItem {
                        label: item.url().to_string(),
                        kind: Some(CompletionItemKind::TEXT),
                        ..CompletionItem::default()
                    })
                    .collect();

                return Ok(Some(CompletionResponse::List(CompletionList {
                    is_incomplete: true,
                    items,
                })));
            }

            let mut parser = self.parser.lock().await;
            if let Some(keyword) = crate::position::retrieve_keyword_at_position(
                doc,
                &mut parser,
                params.text_document_position.position.line as usize,
                params.text_document_position.position.character as usize,
            ) {
                let keyword = keyword.to_ascii_lowercase().replace("-", "_");
                let items = TEMPL_MAP
                    .iter()
                    .filter(|t| t.name.starts_with(&keyword))
                    .map(|t| CompletionItem {
                        label: t.name.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(t.outline.to_string()),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: t.doc.to_string(),
                        })),
                        ..CompletionItem::default()
                    })
                    .collect();

                return Ok(Some(CompletionResponse::List(CompletionList {
                    is_incomplete: true,
                    items,
                })));
            }
        }

        Ok(None)
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let issues = self.update_issues(&params.text_document.uri).await;
        let issues = if let Some(issues) = issues.as_deref() {
            issues
        } else {
            return;
        };
        self.client
            .log_message(MessageType::INFO, format!("Issues: {issues:?}"))
            .await;
        let diagnostics = issues
            .iter()
            .map(|issue| {
                let display_issue = issue.display_issue();
                Diagnostic {
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: display_issue.explanation.clone().unwrap_or_default(),
                    range: issue_line_col_to_range(display_issue),
                    data: serde_json::to_value(issue).ok(),
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();
        self.client
            .log_message(MessageType::INFO, format!("Diagnostics: {diagnostics:?}"))
            .await;
        self.client
            .publish_diagnostics(params.text_document.uri.clone(), diagnostics, None)
            .await;
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        let curr_doc = self.docs.get(&params.text_document.uri);
        if let Some(ref doc) = curr_doc {
            if let Some((Some(issue), range)) = params.context.diagnostics.first().map(|d| {
                (
                    d.data
                        .clone()
                        .and_then(|data| serde_json::from_value::<DIssue>(data).ok()),
                    d.range,
                )
            }) {
                let new_text = doc.full.get_content(Some(range)).replace(
                    issue.content().unwrap_or("<__never_ever__>"),
                    issue
                        .display_issue()
                        .suggestion
                        .as_deref()
                        .unwrap_or("<__invalid_suggestion__>"),
                );
                Ok(Some(vec![CodeActionOrCommand::CodeAction(CodeAction {
                    title: "fix issue".to_string(),
                    edit: Some(WorkspaceEdit {
                        changes: None,
                        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                            edits: vec![OneOf::Left(TextEdit { range, new_text })],
                            text_document: OptionalVersionedTextDocumentIdentifier {
                                uri: params.text_document.uri,
                                version: None,
                            },
                        }])),
                        change_annotations: None,
                    }),
                    ..Default::default()
                })]))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Backend {
    async fn update_issues(&self, uri: &Uri) -> Option<Vec<DIssue>> {
        if let (Some(ref doc), Some(path)) = (self.docs.get(uri), uri.to_file_path()) {
            let page = match doc_from_raw(doc.full.get_content(None).to_string(), path) {
                Ok(doc) => Page::Doc(Arc::new(doc)),
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("{e}"))
                        .await;
                    return None;
                }
            };

            let issues = match get_fixable_issues(&page) {
                Ok(issues) => issues,
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("{e}"))
                        .await;
                    return None;
                }
            };

            Some(issues)
        } else {
            None
        }
    }
}

fn issue_line_col_to_range(display_issue: &DisplayIssue) -> Range {
    Range::new(
        Position::new(
            display_issue.line.unwrap_or(1).saturating_sub(1) as u32,
            display_issue.column.unwrap_or(1).saturating_sub(1) as u32,
        ),
        Position::new(
            display_issue.end_line.unwrap_or(1).saturating_sub(1) as u32,
            display_issue.end_column.unwrap_or(1) as u32,
        ),
    )
}
