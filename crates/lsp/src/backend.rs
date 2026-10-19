// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language server backend

use std::sync::OnceLock;

use crate::{
    config::ServiceConfig,
    processor::{self as mu_processor},
    semantic_tokens::{LEGEND_MODIFIERS, LEGEND_TYPES},
};

use mucad_viewer_ipc::{ViewerProcessInterface, ViewerRequest};
use tower_lsp::{
    Client, LanguageServer, async_trait,
    jsonrpc::Result,
    lsp_types::{
        CompletionOptions, CompletionParams, CompletionResponse, DiagnosticOptions,
        DiagnosticServerCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentDiagnosticParams,
        DocumentDiagnosticReport, DocumentDiagnosticReportPartialResult,
        DocumentDiagnosticReportResult, DocumentFormattingParams, ExecuteCommandParams,
        InitializeParams, InitializeResult, InitializedParams, MessageType, OneOf,
        RelatedFullDocumentDiagnosticReport, SemanticTokensFullOptions, SemanticTokensLegend,
        SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
        SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
        TextDocumentSyncKind, TextEdit, Url,
    },
};

/// LSP service
#[derive(Debug)]
pub struct Service {
    client: Client,
    processor: mu_processor::ProcessorController,
    viewer: OnceLock<ViewerProcessInterface>,
    config: ServiceConfig,
}

impl Service {
    /// New µcad LSP service backend
    pub fn new(
        client: Client,
        processor: mu_processor::ProcessorController,
        config: ServiceConfig,
    ) -> Self {
        Service {
            client,
            processor,
            viewer: OnceLock::new(),
            config,
        }
    }

    fn send_lsp(&self, req: mu_processor::ProcessorRequest) {
        if let Err(err) = self.processor.send_request(req) {
            log::error!("Cannot send request to lsp processor: {err}")
        }
    }
    fn send_viewer(&self, req: ViewerRequest) -> miette::Result<()> {
        if self.config.use_viewer {
            self.viewer
                .get_or_init(|| ViewerProcessInterface::run(true))
                .send_request(req)
                .inspect_err(|err| log::error!("Cannot send request to viewer: {err}"))
        } else {
            Ok(())
        }
    }

    /// Handle active file changed.
    pub async fn on_active_file_changed(&self, params: serde_json::Value) {
        log::info!("on_active_file_changed: {params:?}");
        if let Ok(Some(uri)) = read_uri("uri", &params) {
            self.send_lsp(mu_processor::ProcessorRequest::CompileDocument(uri.clone()));
            match uri.to_file_path() {
                Ok(path) => {
                    log::info!("New active document: {:?}", path);
                    let _ = self.send_viewer(ViewerRequest::ShowSourceCodeFromFile { path });
                }
                Err(()) => log::error!("Cannot parse URI: {uri}"),
            }
        } else {
            log::error!("No 'uri' field in notification parameters");
        }
    }
}

fn read_uri(value: &str, uri_obj: &serde_json::Value) -> std::result::Result<Option<Url>, String> {
    if let Some(external) = uri_obj.get(value).and_then(serde_json::Value::as_str) {
        return Url::parse(external)
            .map_err(|e| {
                log::info!("Failed to parse external URL: {e}");
                format!("Failed to parse external URL: {e}")
            })
            .map(Some);
    }

    Ok(None)
}

fn uri_obj_to_lsp_url(uri_obj: &serde_json::Value) -> std::result::Result<Url, String> {
    if let Some(uri) = read_uri("external", uri_obj)? {
        return Ok(uri);
    }
    if let Some(fs_path) = read_uri("fsPath", uri_obj)? {
        return Ok(fs_path);
    }
    Err("Neither external nor fsPath found in uri object".to_string())
}

#[async_trait]
impl LanguageServer for Service {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        log::info!("initialize");
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions::default(),
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            legend: SemanticTokensLegend {
                                token_types: LEGEND_TYPES.into(),
                                token_modifiers: LEGEND_MODIFIERS.into(),
                            },
                            ..SemanticTokensOptions::default()
                        },
                    ),
                ),
                document_formatting_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    // Paths are completed after `::`.
                    trigger_characters: Some(vec![":".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        log::info!("initialized");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        //log::info!("shutdown");
        let _ = self.send_viewer(ViewerRequest::Exit);
        Ok(())
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri;
        if let Some(last) = params.content_changes.pop() {
            self.send_lsp(mu_processor::ProcessorRequest::ChangeDocument {
                url,
                code: last.text,
            });
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.send_lsp(mu_processor::ProcessorRequest::OpenDocument {
            url: params.text_document.uri,
            code: params.text_document.text,
        });
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        /*self.client
            .log_message(MessageType::INFO, format!("did save: {uri:?}!"))
            .await;
        */
        self.send_lsp(mu_processor::ProcessorRequest::CompileDocument(uri.clone()));
        //let _ = self.send_viewer(ViewerRequest::ShowSourceCodeFromFile { path });
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.send_lsp(mu_processor::ProcessorRequest::CloseDocument(uri))
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let url = params.text_document.uri;
        if let Ok(diag) = self
            .processor
            .request(|reply| mu_processor::ProcessorRequest::GetDocumentDiagnostics(url, reply))
            .await
        {
            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None, // TODO: Get related documents.
                    full_document_diagnostic_report: diag,
                }),
            ))
        } else {
            Ok(DocumentDiagnosticReportResult::Partial(
                DocumentDiagnosticReportPartialResult {
                    related_documents: None,
                },
            ))
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let url = params.text_document.uri;
        Ok(self
            .processor
            .request(|reply| mu_processor::ProcessorRequest::GetFullSemanticTokens(url, reply))
            .await
            .ok()
            .flatten())
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        log::trace!("execute_command: {params:?}");
        match params.command.as_str() {
            "mucad.showPreview" => {
                if let Some(arg) = params.arguments.first() {
                    let uri = match uri_obj_to_lsp_url(match arg.get("uri") {
                        Some(v) => v,
                        None => {
                            log::error!("Missing 'uri' field");
                            return Ok(Some(serde_json::json!({
                                "error": "Missing 'uri' field"
                            })));
                        }
                    }) {
                        Ok(uri) => uri,
                        Err(err) => {
                            return Ok(Some(serde_json::json!({"error": err.to_string()})));
                        }
                    };

                    if let Err(err) = self.send_viewer(ViewerRequest::Restore) {
                        log::error!("Could not send request ViewerRequest::Show: {err}");
                        return Ok(Some(serde_json::json!({
                            "error": "Cannot show viewer: {err}"
                        })));
                    }

                    if let Err(err) = self.send_viewer(ViewerRequest::ShowSourceCodeFromFile {
                        path: uri
                            .to_file_path()
                            .expect("A valid URI containing a file path"),
                    }) {
                        log::error!("{err}");
                        return Ok(Some(serde_json::json!({"error": err.to_string()})));
                    }

                    self.client
                        .log_message(MessageType::INFO, format!("Preview generated for {uri}"))
                        .await;

                    return Ok(Some(serde_json::json!({ "ok": true })));
                }
            }
            "mucad.minimizePreview" => {
                log::info!("MinimizePreview received");
                if let Err(err) = self.send_viewer(ViewerRequest::Minimize) {
                    log::error!("Could not send request ViewerRequest::Minimize: {err}");
                    return Ok(Some(serde_json::json!({
                        "error": "Cannot minimize viewer: {err}"
                    })));
                }
                self.client
                    .log_message(MessageType::INFO, "Preview hidden")
                    .await;
                return Ok(Some(serde_json::json!({ "ok": true })));
            }
            _ => log::info!("Unknown command '{}'", params.command),
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let position = params.text_document_position;
        let url = position.text_document.uri;
        Ok(self
            .processor
            .request(|reply| {
                mu_processor::ProcessorRequest::CompleteDocument(url, position.position, reply)
            })
            .await
            .ok()
            .map(CompletionResponse::Array))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let url = params.text_document.uri;
        match self
            .processor
            .request(|reply| mu_processor::ProcessorRequest::FormatDocument(url, reply))
            .await
        {
            Ok(Ok(edits)) => Ok(Some(edits)),
            // E.g. syntax errors, which are reported as diagnostics.
            Ok(Err(err)) | Err(err) => {
                log::warn!("Cannot format: {err}");
                Ok(None)
            }
        }
    }
}
