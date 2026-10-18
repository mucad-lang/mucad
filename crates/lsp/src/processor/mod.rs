// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language server processor.
//!
//! The processor sets up the µcad language toolchain (parse, resolve, eval).
//! It runs in a separate thread and communication is handled via
//! crossbeam channels with requests and responses.

use mucad_driver::prelude as mu;
use mu::traits::*;

use crossbeam::channel::{Receiver, Sender};

use miette::IntoDiagnostic;

use mu::Url;
use tower_lsp::lsp_types as lsp;

use crate::to_lsp::ToLsp;

/// A processor request.
///
/// Commands that can be passed to the [`Processor`].
#[derive(Clone)]
#[allow(unused, missing_docs)]
pub enum ProcessorRequest {
    AddDocument(Url),
    ChangeDocument {
        url: Url,
        new_code: String,
    },
    RemoveDocument(Url),
    CompileDocument(Url),

    /// Get diagnostics from a document and converts it to LSP diagnostics
    GetDocumentDiagnostics(Url),

    GetFullSemanticTokens(Url),
    FormatDocument(Url),
}

/// A processor response.
pub enum ProcessorResponse {
    /// Error messages and warnings for a specific document received.
    DocumentDiagnostics(Url, lsp::FullDocumentDiagnosticReport),
    /// Output semantic tokens.
    SemanticTokens(Url, lsp::SemanticTokensResult),

    /// A list of edited text snippets, e.g. after a format request.
    TextEdits(Url, Vec<lsp::TextEdit>),

    /// Update the code of the document (e.g. after linting or formatting)
    UpdatedDocumentCode {
        /// Document URL
        url: Url,
        /// The old code of the document
        old: String,
        /// The new code of the document
        new: String,
    },
}

/// The processor  responsible for generating view commands.
///
/// The processor itself runs in a separate thread and can be controlled
/// via [`ProcessorInterface`] by sending requests and handling the corresponding responses.
pub struct Processor {
    /// Request handler.
    pub request_handler: Receiver<ProcessorRequest>,

    /// Response handler.
    pub response_sender: Sender<ProcessorResponse>,

    /// Driver session
    pub session: mu::Session,
}

/// Type alias for a Result from a processor command.
pub type ProcessorResult = miette::Result<Vec<ProcessorResponse>>;

impl Processor {
    /// Handle processor request.
    pub fn handle_request(&mut self, request: ProcessorRequest) -> ProcessorResult {
        match request {
            ProcessorRequest::AddDocument(url) => self.add_document(&url),
            ProcessorRequest::RemoveDocument(url) => self.remove_document(&url),
            ProcessorRequest::CompileDocument(url) => self.compile_document(&url),
            ProcessorRequest::ChangeDocument { url, new_code } => {
                self.change_document(&url, new_code)
            }
            ProcessorRequest::GetDocumentDiagnostics(url) => self.get_document_diagnostics(&url),
            ProcessorRequest::GetFullSemanticTokens(url) => self.get_full_semantic_tokens(&url),
            ProcessorRequest::FormatDocument(url) => self.format_document(&url),
        }
    }
}

/// Request handler implementation (must be private)
impl Processor {
    fn add_document(&mut self, url: &Url) -> ProcessorResult {
        match self.session.load_document(url.clone()) {
            Ok(_) => self.get_document_diagnostics(&url),
            Err(err) => Err(err),
        }
    }

    fn remove_document(&mut self, url: &Url) -> ProcessorResult {
        self.session.remove_document(url);
        Ok(vec![]) // Maybe: Return `ProcessorResponse::RemovedDocument` here?
    }

    fn compile_document(&mut self, url: &Url) -> ProcessorResult {
        self.session.compile_document(url)?;
        self.get_document_diagnostics(url)
    }

    fn change_document(&mut self, url: &Url, new_code: String) -> ProcessorResult {
        self.session.change_document(url, new_code)?;
        self.get_document_diagnostics(url)
    }

    fn get_document_diagnostics(&self, url: &Url) -> ProcessorResult {
        Ok(self
            .session
            .get_document(&url)
            .map(|doc| doc.diags().to_lsp())
            .into_iter()
            .map(|report| ProcessorResponse::DocumentDiagnostics(url.clone(), report))
            .collect())
    }

    fn get_full_semantic_tokens(&self, url: &Url) -> ProcessorResult {
        match self
            .session
            .get_source_file(url)
            .and_then(|doc| doc.ast.as_ref())
        {
            Some(ast) => {
                use crate::semantic_tokens::SemanticTokens;
                let mut ctx = crate::semantic_tokens::TokenContext::new(ast);
                ast.ast.semantic_tokens(&mut ctx);

                Ok(vec![ProcessorResponse::SemanticTokens(
                    url.clone(),
                    lsp::SemanticTokensResult::Tokens(lsp::SemanticTokens {
                        result_id: None, // TODO: support delta updates
                        data: ctx.tokens().clone(),
                    }),
                )])
            }
            None => Ok(vec![]),
        }
    }

    fn format_document(&mut self, url: &Url) -> ProcessorResult {
        match self.session.get_source_file_mut(url) {
            Some(source_file) => {
                let old_source = source_file.source.0.clone();
                source_file.format(&mu::FormatParameters::default())?;

                let text_edits = old_source
                    .compare(&source_file.source)
                    .into_iter()
                    .map(|text_edit| text_edit.to_lsp())
                    .collect();

                Ok(vec![ProcessorResponse::TextEdits(url.clone(), text_edits)])
            }
            None => Err(miette::miette!("No source file at {url}")),
        }
    }
}

/// Send request to the µcad processor and recv requests.
#[derive(Debug)]
pub struct ProcessorController {
    /// Send req interface.
    pub request_sender: Sender<ProcessorRequest>,
    /// Response recv interface.
    pub response_receiver: Receiver<ProcessorResponse>,
}

impl ProcessorController {
    /// Send request.
    pub fn send_request(&self, request: ProcessorRequest) -> miette::Result<()> {
        self.request_sender.send(request).into_diagnostic()
    }

    /// Recv response.
    pub fn recv_response(&self) -> miette::Result<ProcessorResponse> {
        self.response_receiver.recv().into_diagnostic()
    }

    /// Run the processing thread and create interface.
    pub fn run(config: mu::DriverConfig) -> mu::Result<Self> {
        let (request_sender, request_receiver) = crossbeam::channel::unbounded();
        let (response_sender, response_receiver) = crossbeam::channel::unbounded();

        std::thread::spawn(move || {
            let mut processor = Processor {
                request_handler: request_receiver,
                response_sender,
                session: mu::Session::new(config),
            };

            loop {
                if let Ok(request) = processor.request_handler.recv()
                    && let Ok(responses) = processor.handle_request(request)
                {
                    for response in responses {
                        processor.response_sender.send(response).ok();
                    }
                }
            }
        });

        Ok(Self {
            request_sender,
            response_receiver,
        })
    }
}
