// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{commands::SetCode, prelude as mu};

/// A session with a VFS for documents.
pub struct Session {
    /// The cache of documents (virtual FS)
    pub documents: mu::HashMap<mu::Url, mu::Document>,
    /// Render cache
    pub render_cache: mu::RcMut<mu::RenderCache>,
    /// Configuration
    pub config: mu::DriverConfig,
}

impl Session {
    /// Create new session.
    pub fn new(config: mu::DriverConfig) -> Self {
        Self {
            documents: mu::HashMap::default(),
            render_cache: mu::RcMut::new(mu::RenderCache::new()),
            config,
        }
    }

    /// Load a document and try to compile it.
    pub fn load_document(&mut self, url: mu::Url) -> mu::Result<mu::Model> {
        self.documents
            .insert(url.clone(), mu::Document::new(url.clone())?);
        self.compile_document(&url)
    }

    pub fn compile_document(&mut self, url: &mu::Url) -> mu::Result<mu::Model> {
        match self.documents.get_mut(url) {
            Some(document) => document.compile(mu::CompileParameters {
                resolve: mu::ResolveParameters {
                    search_paths: self.config.search_paths.clone(),
                    no_builtin: false,
                },
            }),
            None => Err(miette::miette!("Document `{url}` not found!")),
        }
    }

    /// Remove document. The document is removed when it is physically removed from disk.
    pub fn remove_document(&mut self, url: &mu::Url) -> Option<mu::Document> {
        self.documents.remove(url)
    }

    pub fn get_document(&self, url: &mu::Url) -> Option<&mu::Document> {
        self.documents.get(url)
    }

    pub fn get_source_file(&self, url: &mu::Url) -> Option<&mu::document::SourceFile> {
        self.get_document(url).and_then(|doc| match doc {
            mu::Document::SourceFile(source_file) => Some(source_file.as_ref()),
            _ => None,
        })
    }

    pub fn get_source_file_mut(&mut self, url: &mu::Url) -> Option<&mut mu::document::SourceFile> {
        self.documents.get_mut(url).and_then(|doc| match doc {
            mu::Document::SourceFile(source_file) => Some(source_file.as_mut()),
            _ => None,
        })
    }

    pub fn change_document(&mut self, url: &mu::Url, new_code: String) -> mu::Result<mu::Model> {
        match self.get_source_file_mut(url) {
            Some(source_file) => {
                source_file.set_code(new_code);
                self.compile_document(url)
            }
            None => Err(miette::miette!("Source file does not exist")),
        }
    }

    pub fn format_source_file(&mut self, url: &mu::Url) -> mu::Result<Vec<mu::TextEdit>> {
        match self.get_source_file_mut(url) {
            Some(source_file) => {
                let old_source = source_file.source.clone();
                source_file.format(&mu::FormatParameters::default())?;
                Ok(old_source.compare(&source_file.source))
            }
            None => Ok(vec![]),
        }
    }
}
