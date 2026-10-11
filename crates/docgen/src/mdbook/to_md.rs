// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate markdown for symbols.

use microcad_lang_markdown::{Block, ListItem, Markdown, Section};
use microcad_lang_resolve::{SymbolDef, SymbolNodeExt, SymbolNodeRef, library::symbol};

/// Trait to fetch markdown from a symbol definition.
pub(crate) trait ToMd {
    fn to_md(&self) -> Markdown;
}

/// Documentation text as written, if there is any.
pub(crate) fn doc_block(doc: Option<&str>) -> Option<Block> {
    doc.map(str::trim)
        .filter(|doc| !doc.is_empty())
        .map(|doc| Block::Paragraph(doc.to_string()))
}

impl ToMd for symbol::ParameterList {
    fn to_md(&self) -> Markdown {
        if self.is_empty() {
            return Markdown::default();
        }

        let mut md = Markdown::default();
        md.add_section(Section::new(1, "Parameters", []));
        self.iter().for_each(|param| {
            md.add_section(Section::new(
                2,
                param.id.to_string(),
                doc_block(Some(&param.attr.doc.content)),
            ))
        });
        md
    }
}

impl<'a> ToMd for SymbolNodeRef<'a> {
    fn to_md(&self) -> Markdown {
        use microcad_lang_resolve::library::WorkbenchKind;

        /// Link to the symbol's page with the first line of its documentation.
        fn list_item(symbol: SymbolNodeRef<'_>) -> Option<ListItem> {
            let id = symbol.name()?;
            let filename = match symbol.def {
                SymbolDef::InlineModule(_) | SymbolDef::SourceFile(_) => {
                    format!("./{id}/README.md")
                }
                _ => format!("./{id}.md"),
            };
            let link = format!("[`{id}`]({filename})");
            Some(ListItem::new(
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

        let mut md = Markdown::default();

        if let Some(id) = self.name() {
            md.add_section(Section::new(
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
                Section::list(2, heading, symbols.into_iter().filter_map(list_item))
            {
                md.add_section(section);
            }
        });

        // Functions are documented inline.
        let functions = children(*self, |symbol| {
            matches!(symbol.def(), SymbolDef::Function(_))
        });
        if !functions.is_empty() {
            md.add_section(Section::new(2, "Functions", []));
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
                    .map(|value| ListItem::new(format!("`{id}` = `{value}`"))),
                _ => None,
            });
        if let Some(section) = Section::list(2, "Constants", constants) {
            md.add_section(section);
        }

        // Aliases
        let aliases = self
            .children()
            .filter_map(|symbol| match (symbol.name(), symbol.def()) {
                (Some(id), SymbolDef::Alias(alias)) => {
                    Some(ListItem::new(format!("`{id}` => `{}`", alias.path)))
                }
                _ => None,
            });
        if let Some(section) = Section::list(2, "Aliases", aliases) {
            md.add_section(section);
        }

        md
    }
}
