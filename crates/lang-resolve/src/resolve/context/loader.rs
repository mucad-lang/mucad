// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Load a library

use microcad_lang_base::{HashId, LibraryId, PushIssue, Source, ToCompactString, tree};
use microcad_lang_lower::ir;

use crate::{
    Library, Manifest, ResolveError, ResolveErrorKind, ResolveLibraryContext, ResolveResult,
    library::{Symbol, SymbolNodeId},
    locate,
    resolve::visitor::ResolveVisitor,
};

impl<'lib, 'ctx> ResolveLibraryContext<'lib, 'ctx> {
    /// Load a single file or a library
    pub fn load(&mut self, path: impl AsRef<std::path::Path>) -> ResolveResult<()> {
        let path = path.as_ref();
        if path.is_file() {
            self.load_file(path)?;
        } else {
            self.load_library(path)?;
        };

        ResolveVisitor::new(self).visit();
        Ok(())
    }

    /// Load a single file as library.
    pub fn load_file(&mut self, file_mu: impl AsRef<std::path::Path>) -> ResolveResult<()> {
        if !self.lib().no_std() {
            self.lib_mut().add_std();
        }

        let root = self.lib().root;
        self._load_source(&file_mu, Some(root))?;
        Ok(())
    }

    /// Load a library from a directory containing a `mu.toml` file.
    pub fn load_library(&mut self, lib_path: impl AsRef<std::path::Path>) -> ResolveResult<()> {
        // Locate `lib.mu` in directory
        let lib_mu = locate::lib_mu_path(&lib_path)?;
        log::debug!("Loading library from {lib_mu:?}");

        // Load manifest and load library
        match Manifest::load(locate::mu_toml_path(lib_path)) {
            Ok(manifest) => {
                log::debug!("Loaded manifest:\n{manifest}");
                if !self.lib().no_std() {
                    self.lib_mut().add_std();
                }

                if let Some(deps) = &manifest.dependencies {
                    for (name, dep) in deps.iter() {
                        let lib_id = self.load_external_library(dep.path.as_ref().unwrap())?;
                        self.lib_mut()
                            .dependencies
                            .insert(name.to_compact_string(), lib_id);
                    }
                }

                self.lib_mut().with_manifest(manifest);
            }
            // Error loading manifest or manifest not found, fallback to defaults.
            Err(err) => {
                log::warn!("No manifest:\n{err:?}");
                self.push_err(ResolveError::new(err));
            }
        };
        self._load_source(&lib_mu, None)?;
        Ok(())
    }

    fn load_source(&mut self, path: impl AsRef<std::path::Path>) -> ResolveResult<HashId> {
        let source = Source::load(path)?;
        let id = self
            .ctx()
            .source_cache()
            .with_write(|source_cache| source_cache.insert(source));
        let issues = self.ctx().source_cache().with_read(|source_cache| {
            source_cache
                .get(id)
                .iter()
                .flat_map(|source_unit| source_unit.issues())
                .collect::<Vec<_>>()
        });

        self.append_issues(issues);

        Ok(id)
    }

    /// Load lib with a path of directory containing a `mu.toml` file.
    ///
    /// TODO: Move this to ResolveContext.
    pub fn load_external_library(
        &mut self,
        lib_path: impl AsRef<std::path::Path>,
    ) -> ResolveResult<LibraryId> {
        let mut lib = Library::new();
        let mut context = ResolveLibraryContext::new(&mut lib, self.ctx_mut());
        context.load_library(lib_path)?;
        self.ctx().lib_cache.write_unwrap().insert(lib)
    }

    pub fn _load_source(
        &mut self,
        path: impl AsRef<std::path::Path>,
        parent_id: Option<SymbolNodeId>,
    ) -> ResolveResult<SymbolNodeId> {
        let path = path.as_ref().to_path_buf();
        let id = self.load_source(&path)?;
        println!("Load source file  {path:?}");

        match self.ctx().get_ir(id) {
            // We have a successfully compiled IR, add it to the tree and load file recursively, if necessary.
            Some(ir) => {
                let source_arena = ir.tree.arena();
                let source_node = ir.tree.root();

                // Create matching node in target arena
                let symbol = match &source_node.def {
                    ir::Def::Source(source) => {
                        Symbol::loaded_source_file(source_node.get(), path.clone(), source.clone())
                    }
                    _ => source_node.get().clone().into(),
                };
                let id = if let Some(parent_id) = parent_id {
                    let id = self.arena_mut().new_node(symbol);
                    parent_id.append(id, self.arena_mut());
                    id
                } else {
                    *self.lib_mut().root_mut().get_mut() = symbol;
                    self.lib().root
                };

                // Traverse children recursively
                for child in source_node.children() {
                    let item = child.get();
                    let child_id = match child.get().def {
                        ir::Def::FileModule(_) => {
                            let path = locate::file_module_path(&path, item.name())?;
                            match self._load_source(path, Some(id)) {
                                Ok(id) => id,
                                Err(err) => {
                                    self.push_err(err);
                                    self.arena_mut().new_node(Symbol::root(None))
                                }
                            }
                        }
                        _ => tree::adopt_tree_to_arena(self.arena_mut(), child.id, source_arena),
                    };
                    id.append(child_id, self.arena_mut());
                }

                Ok(id)
            }
            None => Err(ResolveErrorKind::CompileError { path }.into()),
        }
    }
}
