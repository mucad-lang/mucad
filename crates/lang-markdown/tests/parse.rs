// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown parsing tests.

use microcad_lang_markdown::{
    Block, CodeBlock, ListItem, Markdown, OrderedList, ParseError, UnorderedList,
};

fn load(name: &str) -> Markdown {
    Markdown::load(format!("tests/test_cases/{name}.md")).expect("No errors")
}

/// Level and heading of each section.
fn headings(md: &Markdown) -> Vec<(i64, &str)> {
    md.sections()
        .iter()
        .map(|section| (section.level, section.heading.as_str()))
        .collect()
}

/// Display of each test case.
mod display {
    use super::*;

    macro_rules! test_case {
        ($name:ident) => {
            #[test]
            fn $name() {
                let name = stringify!($name);
                insta::assert_snapshot!(name, load(name));
            }
        };
    }

    test_case!(basic);
    test_case!(blocks);
    test_case!(codeblock);
    test_case!(lists);
    test_case!(other_code_blocks);
}

#[test]
fn basic() {
    let md = load("basic");
    assert_eq!(headings(&md), [(1, "Top"), (2, "Sub")]);

    let sections = md.sections();
    assert_eq!(sections[0].content, [Block::Paragraph("Content".into())]);
    assert_eq!(
        sections[1].content,
        [Block::Paragraph("More content".into())]
    );
}

#[test]
fn blocks() {
    let md = load("blocks");
    assert_eq!(
        headings(&md),
        [
            (1, "A test"),
            (2, "Text"),
            (2, "Table"),
            (2, "Code"),
            (2, "Block quote")
        ]
    );

    let sections = md.sections();
    assert!(sections[0].content.is_empty());
    assert_eq!(sections[1].content, [Block::Paragraph("A text".into())]);
    assert!(matches!(sections[2].content.as_slice(), [Block::Table(_)]));
    match sections[3].content.as_slice() {
        [Block::CodeBlock(code_block)] => {
            assert_eq!(code_block.name(), &None);
            assert_eq!(code_block.code(), "Circle(r = 42mm);");
        }
        content => panic!("Expected a code block, got {content:?}"),
    }
    assert!(matches!(
        sections[4].content.as_slice(),
        [Block::BlockQuote(_)]
    ));
}

#[test]
fn codeblock() {
    let md = load("codeblock");
    let code_blocks: Vec<&CodeBlock> = md.code_blocks().collect();

    let names: Vec<_> = code_blocks
        .iter()
        .map(|block| block.name().as_deref())
        .collect();
    assert_eq!(
        names,
        [
            Some("test_empty"),
            Some("test_fail"),
            Some("test_params"),
            Some("test_result_params"),
            Some("multi_line")
        ]
    );

    let fragments: Vec<_> = code_blocks
        .iter()
        .map(|block| block.fragment().as_deref())
        .collect();
    assert_eq!(fragments, [None, Some("fail"), None, Some("fail"), None]);

    let parameters: Vec<_> = code_blocks
        .iter()
        .map(|block| block.header.parameters.clone())
        .collect();
    assert_eq!(
        parameters,
        [
            vec![],
            vec![],
            vec!["hires".to_string()],
            vec!["no_format".to_string()],
            vec![]
        ]
    );

    let codes: Vec<_> = code_blocks.iter().map(|block| block.code()).collect();
    assert_eq!(
        codes,
        [
            "",
            "Circle(r = 42mm);",
            "Rect(r = 42mm);",
            "Rect(r = 42mm);",
            "use std::print;\n\nprint(\"Hello, µcad standard library!\");"
        ]
    );

    // 0-based line of the first code line in the markdown file.
    let line_offsets: Vec<_> = code_blocks.iter().map(|block| block.line_offset).collect();
    assert_eq!(line_offsets, [5, 10, 16, 24, 32]);

    // Test banners belong to their code block and are not kept as paragraphs.
    assert!(md.sections().iter().all(|section| {
        section
            .content
            .iter()
            .all(|block| !matches!(block, Block::Paragraph(text) if text.starts_with("[![test]")))
    }));
}

#[test]
fn other_code_blocks() {
    let md = load("other_code_blocks");

    // Text before the first heading is a section of level 0.
    // `# not a heading` inside the `sh` block is no heading.
    assert_eq!(headings(&md), [(0, ""), (1, "Other code blocks")]);
    assert_eq!(
        md.sections()[0].content,
        [Block::Paragraph("Text before the first heading.".into())]
    );

    // Other code blocks are kept as written.
    let content = &md.sections()[1].content;
    assert_eq!(
        content[0],
        Block::Paragraph("```sh\n# not a heading\necho \"µcad\"\n```".into())
    );

    // Code inside a list item is kept as written, without the item's indentation.
    assert_eq!(
        content[1],
        Block::UnorderedList(UnorderedList::new([ListItem::new("A list item")
            .with_children([Block::Paragraph(
                "```mu,in_list\nCircle(r = 1mm);\n```".into()
            )])]))
    );

    // Only top-level µcad code blocks become code blocks.
    let names: Vec<_> = md
        .code_blocks()
        .map(|block| block.name().as_deref())
        .collect();
    assert_eq!(names, [Some("top_level")]);
}

#[test]
fn lists() {
    let md = load("lists");
    assert_eq!(headings(&md), [(1, "Lists")]);

    assert_eq!(
        md.sections()[0].content,
        [
            Block::UnorderedList(UnorderedList::new([
                ListItem::new("First"),
                ListItem::new("Second with `code`").with_children([Block::UnorderedList(
                    UnorderedList::new([ListItem::new("Nested"), ListItem::new("Nested two")])
                )]),
                ListItem::new("Third"),
            ])),
            Block::OrderedList(OrderedList::new(
                1,
                [
                    ListItem::new("One"),
                    ListItem::new("Two").with_children([Block::OrderedList(OrderedList::new(
                        1,
                        [ListItem::new("Two point one")]
                    ))]),
                    ListItem::new("Three"),
                ]
            )),
            Block::Paragraph("A paragraph between two lists.".into()),
            Block::OrderedList(OrderedList::new(
                5,
                [ListItem::new("Starts at five"), ListItem::new("Six")]
            )),
            Block::UnorderedList(UnorderedList::new([
                ListItem::new("Loose item")
                    .with_children([Block::Paragraph("With a second paragraph.".into())]),
                ListItem::new("Other loose item"),
            ])),
        ]
    );
}

/// Parsing the display output gives the same markdown again.
#[test]
fn lists_round_trip() {
    let md = load("lists");
    let reparsed: Markdown = md.to_string().parse().expect("No errors");
    assert_eq!(reparsed.sections()[0].content, md.sections()[0].content);
}

#[test]
fn unclosed_code_block() {
    let err = "# Circle\n\n```mu\nCircle(r = 4mm);\n\n## Rectangle\n"
        .parse::<Markdown>()
        .unwrap_err();
    assert!(
        matches!(err, ParseError::UnclosedCodeBlock { line: 3 }),
        "{err:?}"
    );
}

#[test]
fn duplicated_code_block_name() {
    let err = "```mu,a\n```\n\n```mu,a\n```\n"
        .parse::<Markdown>()
        .unwrap_err();
    assert!(
        matches!(&err, ParseError::DuplicatedCodeBlockName(name) if name == "a"),
        "{err:?}"
    );
}

#[test]
fn malformed_header() {
    let err = "```mu,a(hires\n```\n".parse::<Markdown>().unwrap_err();
    assert!(matches!(err, ParseError::MalformedHeader), "{err:?}");
}
