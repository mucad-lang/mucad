// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Load a library

use microcad_lang_base::{
    HashId, Identifier, PushIssue, Source, ToCompactString, element::Visibility, tree,
};
use microcad_lang_lower::ir;

use crate::{
    Manifest, ResolveError, ResolveErrorKind, ResolveLibraryContext, ResolveResult,
    library::{Symbol, SymbolNodeId},
    locate,
    resolve::{alias_check::AliasCheck, case_check::CaseCheck, visitor::ResolveVisitor},
};

impl<'lib, 'ctx> ResolveLibraryContext<'lib, 'ctx> {
    /// Load a single file or a library
    ///
    /// Returns `None` if the source could not be compiled. Its issues are recorded.
    pub fn load(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> ResolveResult<Option<SymbolNodeId>> {
        let path = path.as_ref();
        let id = if path.is_file() {
            self.load_file(path)?
        } else {
            self.load_library(path)?
        };

        if id.is_some() {
            self.resolve();
        }
        Ok(id)
    }

    /// Load and resolve a source (e.g. an editor buffer) as library.
    ///
    /// File modules are located relative to the source's path, if it has one.
    ///
    /// Returns `None` if the source could not be compiled. Its issues are recorded.
    pub fn load_source(&mut self, source: Source) -> ResolveResult<Option<SymbolNodeId>> {
        self.add_implicit_dependencies();

        let root = self.lib().root;
        let id = self.load_source_symbol(source, Some(root))?;

        if id.is_some() {
            self.resolve();
        }
        Ok(id)
    }

    /// Resolve the paths of the loaded library, then check its aliases and the case of its symbol names.
    fn resolve(&mut self) {
        ResolveVisitor::new(self).visit();
        AliasCheck::check(self.lib())
            .into_iter()
            .for_each(|err| self.push_err(err));
        CaseCheck::check(self.lib())
            .into_iter()
            .for_each(|warning| self.push_warn(warning));
    }

    /// Load a single file as library.
    ///
    /// Returns `None` if the source could not be compiled. Its issues are recorded.
    pub fn load_file(
        &mut self,
        file_mu: impl AsRef<std::path::Path>,
    ) -> ResolveResult<Option<SymbolNodeId>> {
        self.add_implicit_dependencies();

        let root = self.lib().root;
        self._load_source(&file_mu, Some(root))
    }

    /// Load a library from a directory containing a `mu.toml` file.
    ///
    /// Returns `None` if `lib.mu` could not be compiled. Its issues are recorded.
    pub fn load_library(
        &mut self,
        lib_path: impl AsRef<std::path::Path>,
    ) -> ResolveResult<Option<SymbolNodeId>> {
        // Locate `lib.mu` in directory
        let lib_mu = locate::lib_mu_path(&lib_path)?;
        log::debug!("Loading library from {lib_mu:?}");

        // Load manifest and load library
        match Manifest::load(locate::mu_toml_path(&lib_path)) {
            Ok(manifest) => {
                log::debug!("Loaded manifest:\n{manifest}");

                if let Some(deps) = &manifest.dependencies {
                    for (name, dep) in deps.iter() {
                        // A failed dependency is recorded, its symbols remain unresolved.
                        // Dependency paths are relative to the library.
                        let dep_path = lib_path.as_ref().join(dep.path.as_ref().unwrap());
                        match self.ctx_mut().load_external_library(dep_path) {
                            Ok((lib_id, issues)) => {
                                self.append_issues(issues);
                                self.lib_mut()
                                    .dependencies
                                    .insert(name.to_compact_string(), lib_id);
                            }
                            Err(issues) => self.append_issues(issues),
                        }
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
        // After the manifest is set, which may opt out with `no_std`.
        self.add_implicit_dependencies();

        // The library appears in its own tree under its name, e.g. `std`, so it can refer to
        // itself with `std::geo2d::Circle` or `::std::geo2d::Circle`.
        let root = self.lib().root;
        let Some(id) = self._load_source(&lib_mu, Some(root))? else {
            return Ok(None);
        };
        if let Some(name) = self.lib().name().cloned() {
            let meta = &mut self.arena_mut()[id].get_mut().meta;
            meta.name = Some(Identifier::from(name.as_str()));
            meta.vis = Visibility::Public;
        }
        Ok(Some(id))
    }

    /// Parse and lower a source into the source cache and record its issues.
    fn insert_source(&mut self, source: Source) -> HashId {
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

        id
    }

    /// Load a source file and its file modules into the symbol tree.
    ///
    /// Returns `None` if the source could not be compiled. Its issues are recorded.
    pub fn _load_source(
        &mut self,
        path: impl AsRef<std::path::Path>,
        parent_id: Option<SymbolNodeId>,
    ) -> ResolveResult<Option<SymbolNodeId>> {
        let path = path.as_ref().to_path_buf();
        self.load_source_symbol(Source::load(&path)?, parent_id)
    }

    /// Load a source and its file modules into the symbol tree.
    ///
    /// If `parent_id` is `None`, the source becomes the root of the library.
    /// Returns `None` if the source could not be compiled. Its issues are recorded.
    fn load_source_symbol(
        &mut self,
        source: Source,
        parent_id: Option<SymbolNodeId>,
    ) -> ResolveResult<Option<SymbolNodeId>> {
        let path = source.path();
        let source_name = source.to_string();
        let id = self.insert_source(source);

        match self.ctx().get_ir(id) {
            // We have a successfully compiled IR, add it to the tree and load file recursively, if necessary.
            Some(ir) => {
                let source_arena = ir.tree.arena();
                let source_node = ir.tree.root();

                // Create matching node in target arena
                let symbol = match &source_node.def {
                    ir::Def::Source(source) => Symbol::loaded_source_file(
                        source_node.get(),
                        path.clone().unwrap_or_default(),
                        source.clone(),
                    ),
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
                            // File modules are located relative to the source file.
                            let loaded = match &path {
                                Some(path) => locate::file_module_path(path, item.name())
                                    .map_err(ResolveError::from)
                                    .and_then(|path| {
                                        Ok((self._load_source(&path, Some(id))?, path))
                                    }),
                                None => {
                                    Err(ResolveErrorKind::SourceHasNoPath(source_name.clone())
                                        .into())
                                }
                            };
                            match loaded {
                                Ok((Some(id), _)) => id,
                                // The issues of the file module have been recorded already.
                                Ok((None, path)) => self
                                    .arena_mut()
                                    .new_node(Symbol::failed_source_file(item, path)),
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

                Ok(Some(id))
            }
            // The issues of the source have been recorded by `insert_source`.
            None => Ok(None),
        }
    }
}
