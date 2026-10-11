// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate documentation as markdown book for µcad compiler built-ins.

use std::error::Error;

use microcad_builtin::{
    BuiltinConstant, BuiltinFunction, BuiltinItem, BuiltinModule, BuiltinOperation,
    BuiltinPrimitive,
};
use microcad_lang_markdown::{
    Block, ListItem, Markdown, MdBook, MdBookConfig, MdBookSummary, MdBookSummaryEntry, Section,
};

use crate::{
    BuiltinLibraryDocGenerator, GenerateDoc,
    mdbook::to_md::{ToMd, doc_block},
};

/// A link with a summary: `[name](path): summary`.
fn link_item(name: &str, path: &str, summary: &str) -> ListItem {
    let link = format!("[`{name}`]({path})");
    ListItem::new(match summary {
        "" => link,
        summary => format!("{link}: {summary}"),
    })
}

impl ToMd for BuiltinConstant {
    fn to_md(&self) -> Markdown {
        let value = Block::Paragraph(format!(
            "**{name} = {value}**",
            name = self.info.name,
            value = self.value()
        ));
        Markdown::page(
            self.info.item_name(),
            std::iter::once(value).chain(doc_block(self.info.doc)),
        )
    }
}

impl ToMd for BuiltinFunction {
    fn to_md(&self) -> Markdown {
        let signature = Block::BlockQuote(
            std::iter::once(format!(
                "> `{name}{ty}`",
                name = self.info.name,
                ty = self.ty()
            ))
            .collect(),
        );
        Markdown::page(
            format!("Function {}", self.info.item_name()),
            std::iter::once(signature).chain(doc_block(self.info.doc)),
        )
    }
}

impl ToMd for BuiltinPrimitive {
    fn to_md(&self) -> Markdown {
        Markdown::page(self.info.name, doc_block(self.info.doc))
    }
}

impl ToMd for BuiltinOperation {
    fn to_md(&self) -> Markdown {
        Markdown::page(self.info.name, doc_block(self.info.doc))
    }
}

impl ToMd for BuiltinItem {
    fn to_md(&self) -> Markdown {
        match &self {
            BuiltinItem::Constant(builtin_constant) => builtin_constant.to_md(),
            BuiltinItem::Function(builtin_function) => builtin_function.to_md(),
            BuiltinItem::Primitive(builtin_primitive) => builtin_primitive.to_md(),
            BuiltinItem::Operation(builtin_operation) => builtin_operation.to_md(),
            BuiltinItem::Module(builtin_module) => builtin_module.to_md(),
        }
    }
}

impl ToMd for BuiltinModule {
    fn to_md(&self) -> Markdown {
        let mut md = Markdown::page(self.info.name, doc_block(self.info.doc));

        /// Link to the page of an item in this module.
        fn item(info: &microcad_builtin::BuiltinInfo) -> ListItem {
            let name = info.item_name();
            link_item(name, &format!("{name}.md"), info.doc_summary())
        }

        [
            Section::list(2, "Constants", self.constants().map(|c| item(&c.info))),
            Section::list(2, "Functions", self.functions().map(|f| item(&f.info))),
            Section::list(2, "Primitives", self.primitives().map(|p| item(&p.info))),
            Section::list(2, "Operations", self.operations().map(|o| item(&o.info))),
        ]
        .into_iter()
        .flatten()
        .for_each(|section| md.add_section(section));

        md
    }
}

impl<'builtins> BuiltinLibraryDocGenerator<'builtins> {
    /// The start page with a list of all modules.
    fn index(&self) -> Markdown {
        let mut md = Markdown::page(
            "`__mu`: µcad Compiler built-in library",
            doc_block(Some(microcad_builtin::mu::__MU)),
        );

        let modules = self.lib.modules().map(|module| {
            let name = module.info.item_name();
            link_item(
                name,
                &format!("{name}/README.md"),
                module.info.doc_summary(),
            )
        });
        if let Some(section) = Section::list(2, "Modules", modules) {
            md.add_section(section);
        }

        md
    }

    fn summary(&self) -> MdBookSummary {
        let mut entries = vec![MdBookSummaryEntry::new(0, "__mu", "README.md")];

        self.lib.modules().for_each(|module| {
            let mod_name = module.info.item_name();
            entries.push(MdBookSummaryEntry::new(
                1,
                mod_name,
                format!("{mod_name}/README.md"),
            ));

            module.items.iter().for_each(|item| {
                let item_name = item.info().item_name();
                entries.push(MdBookSummaryEntry::new(
                    2,
                    item_name,
                    format!("{mod_name}/{item_name}.md"),
                ));
            });
        });

        MdBookSummary::from_iter(entries)
    }
}

impl<'builtins> GenerateDoc<MdBook> for BuiltinLibraryDocGenerator<'builtins> {
    fn generate_doc(&self) -> Result<MdBook, Box<dyn Error>> {
        let mut mdbook = MdBook::new(MdBookConfig::default());
        mdbook.config.set_title("µcad built-in library");

        mdbook.add_md("README.md", self.index());

        self.lib.modules().for_each(|module| {
            let mod_name = module.info.item_name();
            mdbook.add_md(format!("{mod_name}/README.md"), module.to_md());

            module.items.iter().for_each(|item| {
                let item_name = item.info().item_name();
                mdbook.add_md(format!("{mod_name}/{item_name}.md"), item.to_md());
            })
        });

        mdbook.add_md("SUMMARY.md", self.summary());

        Ok(mdbook)
    }
}
