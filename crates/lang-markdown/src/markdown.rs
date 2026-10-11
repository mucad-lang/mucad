// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate a single markdown file for symbol.

use std::str::FromStr;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use thiserror::Error;

use crate::{Block, CodeBlock, CodeBlockHeader, ParseError, Section};

#[derive(Error, Debug)]
pub enum MarkdownError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

/// Markdown struct, represented as a linear list of sections.
#[derive(Debug, Default, Clone)]
pub struct Markdown(Vec<Section>);

impl Markdown {
    pub fn new(sections: Vec<Section>) -> Self {
        Self(sections)
    }
}

impl std::str::FromStr for Markdown {
    type Err = ParseError;

    /// Parse markdown into sections, using `pulldown-cmark` events.
    ///
    /// Only top-level blocks become blocks of a section.
    /// Nested content (e.g. code inside lists) is kept as written in its parent block.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut md = Markdown(vec![Section::default()]); // content before the first heading
        let mut depth = 0;
        let mut banner: Option<String> = None;
        let mut code: Option<(CodeBlockHeader, String)> = None;
        let mut code_block_names = std::collections::HashSet::new();

        for (event, range) in Parser::new_ext(text, Options::ENABLE_TABLES).into_offset_iter() {
            let raw = &text[range.clone()];
            match event {
                Event::Start(tag) => {
                    depth += 1;
                    if depth > 1 {
                        continue;
                    }
                    match tag {
                        Tag::Heading { level, .. } => {
                            md.flush_banner(&mut banner);
                            md.add_section(Section {
                                heading: Self::heading_text(raw),
                                level: level as i64,
                                content: Vec::new(),
                            });
                        }
                        Tag::CodeBlock(CodeBlockKind::Fenced(info))
                            if CodeBlockHeader::is_microcad(&info) =>
                        {
                            // `CodeBlockHeader` writes the banner again from the block name.
                            banner = None;
                            let header: CodeBlockHeader = info.parse()?;
                            if let Some(name) = &header.name
                                && !code_block_names.insert(name.clone())
                            {
                                return Err(ParseError::DuplicatedCodeBlockName(name.clone()));
                            }
                            code = Some((header, String::new()));
                        }
                        Tag::Paragraph if CodeBlockHeader::is_test_banner(raw) => {
                            md.flush_banner(&mut banner);
                            banner = Some(raw.trim_end().to_string());
                        }
                        Tag::Table(_) => md.push_block(&mut banner, Block::Table(Self::lines(raw))),
                        Tag::BlockQuote(_) => {
                            md.push_block(&mut banner, Block::BlockQuote(Self::lines(raw)))
                        }
                        // Paragraphs, lists, other code blocks, HTML: preserved as written.
                        _ => md.push_block(&mut banner, Block::Paragraph(raw.trim_end().to_string())),
                    }
                }
                Event::Text(text) if depth == 1 => {
                    if let Some((_, code)) = &mut code {
                        code.push_str(&text);
                    }
                }
                Event::End(TagEnd::CodeBlock) if depth == 1 && code.is_some() => {
                    depth -= 1;
                    let line_index = Self::line_index(text, range.start);
                    if !Self::is_closed(raw) {
                        return Err(ParseError::UnclosedCodeBlock {
                            line: line_index + 1,
                        });
                    }
                    if let Some((header, mut code)) = code.take() {
                        if code.ends_with('\n') {
                            code.pop();
                        }
                        md.push_block(
                            &mut banner,
                            Block::CodeBlock(CodeBlock {
                                header,
                                code,
                                line_offset: line_index + 1,
                            }),
                        );
                    }
                }
                Event::End(_) => depth -= 1,
                // Top-level leaf, e.g. `---`
                Event::Rule if depth == 0 => {
                    md.push_block(&mut banner, Block::Paragraph(raw.trim_end().to_string()))
                }
                _ => {}
            }
        }
        md.flush_banner(&mut banner);

        // No content before the first heading.
        if md.0.first().is_some_and(Section::is_empty) {
            md.0.remove(0);
        }

        Ok(md)
    }
}

/// Parse helpers
impl Markdown {
    /// Push a block into the current section, keeping a pending test banner as text.
    fn push_block(&mut self, banner: &mut Option<String>, block: Block) {
        self.flush_banner(banner);
        self.current_section_mut().content.push(block);
    }

    /// A test banner which is not followed by a µcad code block is kept as paragraph.
    fn flush_banner(&mut self, banner: &mut Option<String>) {
        if let Some(banner) = banner.take() {
            self.current_section_mut()
                .content
                .push(Block::Paragraph(banner));
        }
    }

    fn current_section_mut(&mut self) -> &mut Section {
        self.0.last_mut().expect("A section")
    }

    /// `## Title` → `Title`, setext `Title\n===` → `Title`
    fn heading_text(raw: &str) -> String {
        let first_line = raw.lines().next().unwrap_or_default().trim();
        first_line
            .trim_start_matches('#')
            .trim()
            .trim_end_matches('#')
            .trim()
            .to_string()
    }

    fn lines<T: FromIterator<String>>(raw: &str) -> T {
        raw.lines().map(String::from).collect()
    }

    /// 0-based line index of a byte position.
    fn line_index(text: &str, pos: usize) -> usize {
        text[..pos].matches('\n').count()
    }

    /// Returns true if a fenced code block ends with a closing fence.
    ///
    /// The closing fence must use the same character and be at least as long as the opening one.
    fn is_closed(raw: &str) -> bool {
        let mut lines = raw.lines();
        let Some(opening) = lines.next().map(str::trim_start) else {
            return false;
        };
        let marker = opening.chars().next().unwrap_or('`');
        let fence_len = opening.chars().take_while(|c| *c == marker).count();

        lines.last().map(str::trim).is_some_and(|closing| {
            closing.chars().all(|c| c == marker) && closing.chars().count() >= fence_len
        })
    }

    /// The sections of this markdown.
    pub fn sections(&self) -> &[Section] {
        &self.0
    }
}

impl Markdown {
    /// Create a Markdown instance from any type that implements Display.
    pub fn from_display(val: impl std::fmt::Display) -> Self {
        crate::md!("{val}")
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, MarkdownError> {
        let input = std::fs::read_to_string(path)?;
        Markdown::from_str(&input).map_err(|err| err.into())
    }

    /// Add a new section.
    pub fn add_section(&mut self, section: Section) {
        self.0.push(section)
    }

    /// Nest another markdown
    pub fn nest(&mut self, md: Markdown, n: i64) {
        self.0.extend(md.0.into_iter().map(|s| s.nested(n)));
    }

    /// Returns an iterator over all code blocks in the entire document.
    pub fn code_blocks(&self) -> impl Iterator<Item = &CodeBlock> {
        self.0
            .iter() // Iterate over Vec<Section>
            .flat_map(|section| section.content.iter()) // Flatten Blocks
            .filter_map(|block| {
                if let Block::CodeBlock(block) = block {
                    Some(block)
                } else {
                    None
                }
            })
    }

    /// Returns an mut iterator over all code blocks in the entire document.
    pub fn code_blocks_mut(&mut self) -> impl Iterator<Item = &mut CodeBlock> {
        self.0
            .iter_mut() // Iterate over Vec<Section>
            .flat_map(|section| section.content.iter_mut()) // Flatten Blocks
            .filter_map(|block| {
                if let Block::CodeBlock(block) = block {
                    Some(block)
                } else {
                    None
                }
            })
    }
}

impl std::fmt::Display for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, section) in self.0.iter().enumerate() {
            // Add a newline between sections for readability,
            // but not before the very first one.
            if i > 0 && !section.is_empty() {
                writeln!(f)?;
            }
            write!(f, "{section}")?;
        }
        Ok(())
    }
}
