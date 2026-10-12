// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate a markdown book from a symbol tree.

use microcad_lang_markdown as md;
use microcad_lang_resolve::{SymbolDef, SymbolNodeExt, SymbolNodeRef, library::symbol};

use crate::{
    GenerateDoc, LibraryDocGenerator,
    mdbook::{ToMd, doc_block},
};

impl ToMd for symbol::ParameterList {
    fn to_md(&self) -> md::Markdown {
        if self.is_empty() {
            return md::Markdown::default();
        }

        let mut md = md::Markdown::page("Parameters", []);
        self.iter().for_each(|param| {
            md.add_section(md::Section::new(
                2,
                param.id.to_string(),
                doc_block(Some(&param.attr.doc.content)),
            ))
        });
        md
    }
}

impl<'a> ToMd for SymbolNodeRef<'a> {
    fn to_md(&self) -> md::Markdown {
        use microcad_lang_resolve::library::WorkbenchKind;

        /// Link to the symbol's page with the first line of its documentation.
        fn list_item(symbol: SymbolNodeRef<'_>) -> Option<md::ListItem> {
            let id = symbol.name()?;
            let filename = match symbol.def {
                SymbolDef::InlineModule(_) | SymbolDef::SourceFile(_) => {
                    format!("./{id}")
                }
                _ => format!("./{id}.md"),
            };
            let link = format!("[`{id}`]({filename})");
            Some(md::ListItem::new(
                match symbol.doc().and_then(|doc| doc.lines().next()) {
                    Some(summary) => format!("{link}: {summary}"),
                    None => link,
                },
            ))
        }

        /// Public children of the symbol, matching `p`.
        fn children<'a>(
            symbol: SymbolNodeRef<'a>,
            p: impl FnMut(&SymbolNodeRef<'a>) -> bool,
        ) -> Vec<SymbolNodeRef<'a>> {
            symbol
                .children()
                .filter(|symbol| symbol.is_public())
                .filter(p)
                .collect()
        }

        fn is_workbench(symbol: &SymbolNodeRef<'_>, kind: WorkbenchKind) -> bool {
            matches!(symbol.def(), SymbolDef::Workbench(workbench) if workbench.signature.kind == kind)
        }

        let mut md = md::Markdown::default();

        if let Some(id) = self.name() {
            md.add_section(md::Section::new(
                1,
                id.to_string(),
                doc_block(self.doc().map(String::as_str)),
            ));
        }

        // Lists of symbols with their own pages.
        let lists = [
            (
                "Sub-modules",
                children(*self, |symbol| {
                    matches!(
                        symbol.def(),
                        SymbolDef::InlineModule(_) | SymbolDef::SourceFile(_)
                    )
                }),
            ),
            (
                "Sketches",
                children(*self, |symbol| is_workbench(symbol, WorkbenchKind::Sketch)),
            ),
            (
                "Parts",
                children(*self, |symbol| is_workbench(symbol, WorkbenchKind::Part)),
            ),
            (
                "Operations",
                children(*self, |symbol| is_workbench(symbol, WorkbenchKind::Op)),
            ),
        ];
        lists.into_iter().for_each(|(heading, symbols)| {
            if let Some(section) =
                md::Section::list(2, heading, symbols.into_iter().filter_map(list_item))
            {
                md.add_section(section);
            }
        });

        // Functions are documented inline.
        let functions = children(*self, |symbol| {
            matches!(symbol.def(), SymbolDef::Function(_))
        });
        if !functions.is_empty() {
            md.add_section(md::Section::new(2, "Functions", []));
            functions
                .iter()
                .for_each(|function| md.nest(function.to_md(), 2));
        }

        // Constants
        let constants = self
            .children()
            .filter_map(|symbol| match (symbol.name(), symbol.def()) {
                (Some(id), SymbolDef::Constant(constant)) => constant
                    .value()
                    .map(|value| md::ListItem::new(format!("`{id}` = `{value}`"))),
                _ => None,
            });
        if let Some(section) = md::Section::list(2, "Constants", constants) {
            md.add_section(section);
        }

        // Aliases
        let aliases = self
            .children()
            .filter_map(|symbol| match (symbol.name(), symbol.def()) {
                (Some(id), SymbolDef::Alias(alias)) => {
                    Some(md::ListItem::new(format!("`{id}` => `{}`", alias.path)))
                }
                _ => None,
            });
        if let Some(section) = md::Section::list(2, "Aliases", aliases) {
            md.add_section(section);
        }

        md
    }
}

impl<'lib> LibraryDocGenerator<'lib> {
    /// Return the path for a symbol, relative to the source folder of the book.
    ///
    /// For example `std::geo2d::Circle` returns `geo2d/Circle.md`.
    fn symbol_path(symbol: SymbolNodeRef<'_>) -> std::path::PathBuf {
        let path: std::path::PathBuf = symbol
            .abs_path()
            .iter()
            .skip(1)
            .map(|id| id.to_string())
            .collect();
        match symbol.def {
            SymbolDef::SourceFile(..) | SymbolDef::InlineModule(..) => path.join("README.md"),
            _ => path.with_extension("md"),
        }
    }

    /// Returns true if the symbol has its own page.
    fn has_page(symbol: &SymbolNodeRef<'_>) -> bool {
        matches!(
            symbol.def(),
            SymbolDef::SourceFile(_) | SymbolDef::InlineModule(_) | SymbolDef::Workbench(_)
        )
    }

    /// Title of the library: its name from the manifest or the name of its main source file.
    fn title(&self) -> String {
        self.lib
            .name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "Library".to_string())
    }

    /// Summary entries of a symbol and its public modules and workbenches.
    fn summary_entries(
        symbol: SymbolNodeRef<'_>,
        name: String,
        level: usize,
        entries: &mut Vec<md::MdBookSummaryEntry>,
    ) {
        entries.push(md::MdBookSummaryEntry::new(
            level,
            name,
            Self::symbol_path(symbol),
        ));

        let children: Vec<_> = symbol
            .children()
            .filter(|child| child.is_public() && Self::has_page(child))
            .collect();

        // Modules first, then workbenches.
        let (modules, workbenches): (Vec<_>, Vec<_>) = children
            .into_iter()
            .partition(|child| !matches!(child.def(), SymbolDef::Workbench(_)));

        modules.into_iter().chain(workbenches).for_each(|child| {
            if let Some(id) = child.name() {
                Self::summary_entries(child, id.to_string(), level + 1, entries)
            }
        });
    }

    fn summary(&self) -> md::MdBookSummary {
        let mut entries = Vec::new();
        Self::summary_entries(self.lib.root(), self.title(), 0, &mut entries);
        md::MdBookSummary::from_iter(entries)
    }
}

impl<'lib> GenerateDoc<md::MdBook> for LibraryDocGenerator<'lib> {
    fn generate_doc(&self) -> Result<md::MdBook, Box<dyn std::error::Error>> {
        let mut mdbook = md::MdBook::new(md::MdBookConfig::default());
        mdbook.config.set_title(self.title());

        // Functions are documented inline in their module's page.
        self.lib
            .root()
            .descendants()
            .filter(Self::has_page)
            .for_each(|symbol| mdbook.add_md(Self::symbol_path(symbol), symbol.to_md()));

        mdbook.add_md("SUMMARY.md", self.summary());

        Ok(mdbook)
    }
}
