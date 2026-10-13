// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A tool to extract snippets from a markdown or mdbook into a list of µcad source files.
//!
//! This tool is also supposed to be an example on how to use the microcad-lang-markdown API.
//! Hence, we intentionally do *NOT* use the microcad-driver API here.

use clap::Parser;

use microcad_lang_markdown as md;
use miette::IntoDiagnostic;

/// µcad md extractor command line interface
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct ExtractCli {
    /// Input markdown file `*.md` or `book.toml` of an mdbook
    input: std::path::PathBuf,

    /// Destination directory (uses current directory be default)
    output: Option<std::path::PathBuf>,
}

/// Converts codesnipped path and name into a flattened `.mu`
///
/// types/collections/list/range.md#some_test -> types_collections_list_range_some_test.mu
pub fn mu_base_path(input_path: impl AsRef<std::path::Path>, name: &str) -> std::path::PathBuf {
    let input_path = input_path.as_ref();
    let mut base = std::path::PathBuf::new();
    if let Some(parent) = input_path.parent() {
        base.push(parent);
    }
    if let Some(stem) = input_path.file_stem() {
        base.push(stem);
    }

    format!(
        "{base}_{name}.{ext}",
        base = base
            .to_str()
            .expect("A valid path")
            .replace('/', "_")
            .replace('.', "_")
            .replace('#', "_"),
        ext = microcad_lang_base::MUCAD_EXTENSION,
    )
    .into()
}

fn main() -> miette::Result<()> {
    let cli = ExtractCli::parse();

    let mdbook = md::MdBook::load(cli.input).into_diagnostic()?;
    let output = match cli.output {
        Some(path) => path,
        None => std::env::current_dir().into_diagnostic()?,
    };

    mdbook
        .code_blocks()
        .try_for_each(|(input_path, code_block)| {
            let Some(name) = code_block.name() else {
                println!("❌ {} => No output: unnamed snippet", input_path.display());
                return Ok(());
            };
            let input = input_path.display();
            let output_path = output.join(mu_base_path(&input_path, name));

            let content = format!(
                "// Extracted from: {input}\n// {}\n\n{}\n",
                md::CodeBlockHeader::test_banner_string(name),
                code_block.code()
            );
            std::fs::write(&output_path, content).into_diagnostic()?;

            println!(
                "✅ {input}#{name} ➡️  {output}",
                output = output_path.display()
            );

            Ok(())
        })
}
