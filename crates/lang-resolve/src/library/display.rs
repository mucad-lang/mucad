// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::{DisplayOneLine, Stability};
use microcad_lang_lower::ir::Visibility;

use crate::{
    Library, Manifest, SymbolDef,
    library::{SourceFile, Symbol},
};

impl std::fmt::Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string_pretty(&self).unwrap_or_default())
    }
}

impl DisplayOneLine for Manifest {
    fn to_string_one_line(&self) -> String {
        format!(
            "{name} {version}",
            name = self.library.name,
            version = self.library.version
        )
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root())
    }
}

impl DisplayOneLine for Library {}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.to_string_one_line())
    }
}

impl DisplayOneLine for Symbol {
    fn to_string_one_line(&self) -> String {
        format!(
            "{name}{def}{ver}{vis}{doc}",
            name = {
                match &self.meta.name {
                    Some(name) => format!("{name}: "),
                    None => String::new(),
                }
            },
            def = self.def.to_string_one_line(),
            ver = match &self.ver.stability {
                Stability::Experimental => "🧪 ",
                Stability::Deprecated { .. } => "⚠️ ",
                Stability::Stable => "",
            },
            vis = match &self.meta.vis {
                Visibility::Public => " [pub]",
                Visibility::Private => "",
            },
            doc = {
                let doc = &self.doc.to_string_one_line();
                if doc.is_empty() {
                    String::new()
                } else {
                    format!(" {doc}")
                }
            }
        )
    }
}

impl std::fmt::Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_one_line())
    }
}

impl DisplayOneLine for SourceFile {
    fn to_string_one_line(&self) -> String {
        match self {
            SourceFile::NotLoaded => "NotLoaded".to_string(),
            SourceFile::Loaded { path, .. } => format!("Loaded({})", path.display()),
            SourceFile::Failed { path } => format!("Failed({})", path.display()),
        }
    }
}

impl std::fmt::Display for SymbolDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into(); // "Function"
        write!(f, "{name}")?;

        let def = match self {
            SymbolDef::Root(Some(source_file)) => source_file.to_string_one_line(),
            SymbolDef::SourceFile(file_module) => file_module.to_string_one_line(),
            SymbolDef::Workbench(workbench) => workbench.to_string_one_line(),
            SymbolDef::Function(function) => function.to_string_one_line(),
            SymbolDef::Constant(constant) => constant.to_string_one_line(),
            SymbolDef::Alias(alias) => alias.to_string_one_line(),
            SymbolDef::Wildcard(wildcard) => wildcard.to_string_one_line(),
            _ => String::new(),
        };

        if def.is_empty() {
            Ok(())
        } else {
            write!(f, " {def}",)
        }
    }
}

impl DisplayOneLine for SymbolDef {}
