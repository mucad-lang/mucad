// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language server processor.
//!
//! The processor compiles the open documents with the µcad toolchain.
//! It runs in a separate thread and is controlled with requests over a channel,
//! requests with a result send it over their own reply channel.

use mucad_driver::prelude as mu;

use crossbeam::channel::{Receiver, Sender};
use miette::IntoDiagnostic;
use tokio::sync::oneshot;

use mu::Url;
use tower_lsp::lsp_types as lsp;

use crate::to_lsp::ToLsp;

/// The channel to send the result of a request.
pub type Reply<T> = oneshot::Sender<T>;

/// A processor request.
pub enum ProcessorRequest {
    /// Open a document with its code and compile it.
    OpenDocument { url: Url, code: String },
    /// Change the code of a document and compile it.
    ChangeDocument { url: Url, code: String },
    /// Close a document.
    CloseDocument(Url),
    /// Compile a document again, e.g. after its file modules have been saved.
    CompileDocument(Url),

    /// Get the diagnostics of the last compilation of a document.
    GetDocumentDiagnostics(Url, Reply<lsp::FullDocumentDiagnosticReport>),
    /// Get the semantic tokens of a document, `None` if it has syntax errors.
    GetFullSemanticTokens(Url, Reply<Option<lsp::SemanticTokensResult>>),
    /// Format a document into text edits.
    FormatDocument(Url, Reply<miette::Result<Vec<lsp::TextEdit>>>),
}

/// An open document.
struct Document {
    source: mu::Source,
    /// The diagnostics of the last compilation.
    diagnostics: mu::Diagnostics,
}

/// The processor, which compiles the open documents.
struct Processor {
    compiler: mu::Compiler,
    documents: mu::HashMap<Url, Document>,
}

impl Processor {
    fn handle_request(&mut self, request: ProcessorRequest) {
        // A reply fails if the receiver has gone, e.g. a cancelled request.
        match request {
            ProcessorRequest::OpenDocument { url, code }
            | ProcessorRequest::ChangeDocument { url, code } => {
                let source = mu::Source::new(mu::SourceKind::Url(url.clone()), code);
                self.documents.insert(
                    url.clone(),
                    Document {
                        source,
                        diagnostics: Default::default(),
                    },
                );
                self.compile_document(&url);
            }
            ProcessorRequest::CloseDocument(url) => {
                self.documents.remove(&url);
            }
            ProcessorRequest::CompileDocument(url) => self.compile_document(&url),
            ProcessorRequest::GetDocumentDiagnostics(url, reply) => {
                reply.send(self.document_diagnostics(&url)).ok();
            }
            ProcessorRequest::GetFullSemanticTokens(url, reply) => {
                reply.send(self.full_semantic_tokens(&url)).ok();
            }
            ProcessorRequest::FormatDocument(url, reply) => {
                reply.send(self.format_document(&url)).ok();
            }
        }
    }

    fn compile_document(&mut self, url: &Url) {
        if let Some(document) = self.documents.get_mut(url) {
            document.diagnostics = match self.compiler.compile(document.source.clone()) {
                Ok((_, diagnostics)) | Err(diagnostics) => diagnostics,
            };
        }
    }

    fn document_diagnostics(&self, url: &Url) -> lsp::FullDocumentDiagnosticReport {
        self.documents
            .get(url)
            .map(|document| document.diagnostics.to_lsp(&document.source))
            .unwrap_or_default()
    }

    fn full_semantic_tokens(&self, url: &Url) -> Option<lsp::SemanticTokensResult> {
        let document = self.documents.get(url)?;
        let (ast, _) = mu::parse(&document.source).ok()?;
        Some(lsp::SemanticTokensResult::Tokens(lsp::SemanticTokens {
            result_id: None, // TODO: support delta updates
            data: crate::semantic_tokens::semantic_tokens(&document.source, ast.tree()),
        }))
    }

    fn format_document(&self, url: &Url) -> miette::Result<Vec<lsp::TextEdit>> {
        let document = self
            .documents
            .get(url)
            .ok_or_else(|| miette::miette!("Document `{url}` is not open"))?;
        let formatted = mu::Formatter::new()
            .format(document.source.clone())
            .map_err(|_| miette::miette!("Document `{url}` has syntax errors"))?;
        Ok(document
            .source
            .compare(&formatted.source)
            .iter()
            .map(|edit| edit.to_lsp(&document.source))
            .collect())
    }
}

/// Sends requests to the µcad processor thread.
#[derive(Debug)]
pub struct ProcessorController {
    request_sender: Sender<ProcessorRequest>,
}

impl ProcessorController {
    /// Send a request without result.
    pub fn send_request(&self, request: ProcessorRequest) -> miette::Result<()> {
        self.request_sender.send(request).into_diagnostic()
    }

    /// Send a request and wait for its result.
    ///
    /// `request` creates the request with the channel for the result.
    pub async fn request<T>(
        &self,
        request: impl FnOnce(Reply<T>) -> ProcessorRequest,
    ) -> miette::Result<T> {
        let (reply, result) = oneshot::channel();
        self.send_request(request(reply))?;
        result.await.into_diagnostic()
    }

    /// Run the processor thread, which compiles with the settings of `config`.
    pub fn run(config: mu::DriverConfig) -> mu::Result<Self> {
        let (request_sender, request_receiver): (_, Receiver<ProcessorRequest>) =
            crossbeam::channel::unbounded();

        std::thread::spawn(move || {
            let mut processor = Processor {
                compiler: mu::Compiler::from_config(&config),
                documents: Default::default(),
            };
            // Ends when the controller has been dropped.
            for request in request_receiver {
                processor.handle_request(request);
            }
        });

        Ok(Self { request_sender })
    }
}
