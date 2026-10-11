// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate a markdown book from a symbol tree.

mod builtin;
mod to_md;

use std::error::Error;

use microcad_lang_markdown::{MdBook, MdBookConfig, MdBookSummary, MdBookSummaryEntry};
use microcad_lang_resolve::{SymbolDef, SymbolNodeExt, SymbolNodeRef};

use crate::{GenerateDoc, LibraryDocGenerator, mdbook::to_md::ToMd};

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

    /// Top-level symbols with a page, e.g. the main source file module.
    ///
    /// The library root itself is an unnamed symbol without a page.
    fn top_level(&self) -> Vec<SymbolNodeRef<'lib>> {
        self.lib.root().children().filter(Self::has_page).collect()
    }

    /// Title of the library: its name from the manifest or the name of its main source file.
    fn title(&self) -> String {
        self.lib
            .name()
            .map(|name| name.to_string())
            .or_else(|| {
                self.top_level()
                    .into_iter()
                    .find_map(|symbol| symbol.name().map(|id| id.to_string()))
            })
            .unwrap_or_else(|| "Library".to_string())
    }

    /// Summary entries of a symbol and its public modules and workbenches.
    fn summary_entries(
        symbol: SymbolNodeRef<'_>,
        name: String,
        level: usize,
        entries: &mut Vec<MdBookSummaryEntry>,
    ) {
        entries.push(MdBookSummaryEntry::new(
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

    fn summary(&self) -> MdBookSummary {
        let mut entries = Vec::new();
        self.top_level().into_iter().for_each(|symbol| {
            if let Some(id) = symbol.name() {
                Self::summary_entries(symbol, id.to_string(), 0, &mut entries)
            }
        });
        MdBookSummary::from_iter(entries)
    }
}

impl<'lib> GenerateDoc<MdBook> for LibraryDocGenerator<'lib> {
    fn generate_doc(&self) -> Result<MdBook, Box<dyn Error>> {
        let mut mdbook = MdBook::new(MdBookConfig::default());
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
