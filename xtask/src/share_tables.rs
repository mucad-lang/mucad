// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate the Huffman tables of shared links from the tokens of µcad files.
//!
//! The code of a link is a sequence of tokens, each encoded with the table of the token kinds.
//! The text of a token, e.g. of an identifier, is encoded with the table of the characters,
//! followed by the end of the text.

use std::collections::{BTreeMap, BinaryHeap};
use std::error::Error;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use mucad_driver::prelude as mu;

/// Directories, which are not searched for µcad files.
const EXCLUDED_DIRS: &[&str] = &["target", "node_modules", "dist", "pkg", ".git"];

/// The kind of tokens of other kinds in the table of token kinds.
const OTHER_TOKEN: &str = "Other";

/// A symbol of the character table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TextSymbol {
    /// A character of a text.
    Char(char),
    /// The end of a text.
    End,
    /// A character, which is not in the table, followed by its code point.
    Other,
}

/// The number of occurrences of the token kinds and of the characters of their texts.
#[derive(Default)]
pub struct Statistics {
    /// The number of lexed files.
    pub files: usize,
    /// The number of tokens of each kind, e.g. `Identifier` or `Keyword(Fn)`.
    pub tokens: BTreeMap<String, usize>,
    /// The texts of the kinds of tokens with a fixed text, e.g. `fn` of `Keyword(Fn)`.
    fixed: BTreeMap<String, String>,
    /// The number of each character in the texts of the tokens and of the ends of the texts.
    texts: BTreeMap<TextSymbol, usize>,
}

impl Statistics {
    /// Count the tokens of the µcad `code`.
    pub fn add(&mut self, code: &str) {
        self.files += 1;
        for token in mu::share::shared_tokens(code) {
            *self.tokens.entry(token.kind.clone()).or_default() += 1;
            if token.fixed {
                self.fixed.insert(token.kind, token.text);
            } else {
                for c in token.text.chars() {
                    *self.texts.entry(TextSymbol::Char(c)).or_default() += 1;
                }
                *self.texts.entry(TextSymbol::End).or_default() += 1;
            }
        }
        // Tokens and characters, which are not in the tables, can be encoded too.
        self.tokens.entry(OTHER_TOKEN.into()).or_insert(1);
        self.texts.entry(TextSymbol::Other).or_insert(1);
    }

    /// The number of tokens.
    pub fn token_count(&self) -> usize {
        self.tokens.values().sum()
    }

    /// The number of characters of the texts of the tokens.
    pub fn char_count(&self) -> usize {
        self.texts
            .iter()
            .filter(|(symbol, _)| matches!(symbol, TextSymbol::Char(_)))
            .map(|(_, count)| count)
            .sum()
    }
}

/// The µcad files in `paths`, which are files or directories to search.
pub fn mucad_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files: Vec<_> = paths
        .iter()
        .flat_map(|path| {
            walkdir::WalkDir::new(path)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|entry| {
                    entry.depth() == 0
                        || !EXCLUDED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref())
                })
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.path().extension().is_some_and(|ext| {
                            mu::MUCAD_EXTENSIONS.contains(&&*ext.to_string_lossy())
                        })
                })
                .map(|entry| entry.into_path())
        })
        .collect();
    files.dedup();
    files
}

/// The lengths of the Huffman codes of symbols with `counts`, in the order of the counts.
fn code_lengths(counts: &[usize]) -> Vec<u8> {
    if counts.len() == 1 {
        return vec![1];
    }
    // A node is a symbol or two merged nodes, the smallest count is merged first, for equal
    // counts the earlier node, so the codes do not depend on the order of the heap.
    let mut parents: Vec<Option<usize>> = vec![None; counts.len()];
    let mut heap: BinaryHeap<_> = counts
        .iter()
        .enumerate()
        .map(|(node, count)| std::cmp::Reverse((*count, node)))
        .collect();
    while let (Some(std::cmp::Reverse((a, left))), Some(std::cmp::Reverse((b, right)))) =
        (heap.pop(), heap.pop())
    {
        let node = parents.len();
        parents.push(None);
        parents[left] = Some(node);
        parents[right] = Some(node);
        heap.push(std::cmp::Reverse((a + b, node)));
    }
    (0..counts.len())
        .map(|mut node| {
            let mut length = 0;
            while let Some(parent) = parents[node] {
                node = parent;
                length += 1;
            }
            length
        })
        .collect()
}

/// The canonical Huffman codes of symbols with `counts`: the code and its number of bits,
/// in the order of the counts.
///
/// Shorter codes come first, codes of the same length in the order of the symbols, so the
/// codes follow from their lengths.
fn canonical_codes(counts: &[usize]) -> Vec<(u32, u8)> {
    let lengths = code_lengths(counts);
    let mut order: Vec<_> = (0..counts.len()).collect();
    order.sort_by_key(|&symbol| (lengths[symbol], symbol));

    let mut codes = vec![(0, 0); counts.len()];
    let mut code = 0u32;
    let mut previous = lengths[order[0]];
    for symbol in order {
        code <<= lengths[symbol] - previous;
        previous = lengths[symbol];
        codes[symbol] = (code, previous);
        code += 1;
    }
    codes
}

/// A table row: the symbol in Rust, its count and its code.
struct Row {
    symbol: String,
    count: usize,
    code: u32,
    bits: u8,
}

/// The rows of a table of symbols with counts, ordered by their codes.
fn rows(symbols: impl IntoIterator<Item = (String, usize)>) -> Vec<Row> {
    let (symbols, counts): (Vec<_>, Vec<_>) = symbols.into_iter().unzip();
    let mut rows: Vec<_> = symbols
        .into_iter()
        .zip(counts.iter())
        .zip(canonical_codes(&counts))
        .map(|((symbol, count), (code, bits))| Row {
            symbol,
            count: *count,
            code,
            bits,
        })
        .collect();
    rows.sort_by_key(|row| (row.bits, row.code));
    rows
}

/// The average number of bits per symbol.
fn average_bits(rows: &[Row]) -> f64 {
    let total: usize = rows.iter().map(|row| row.count).sum();
    let bits: usize = rows.iter().map(|row| row.count * row.bits as usize).sum();
    bits as f64 / total as f64
}

/// Print the rows with their probabilities.
fn print_rows(title: &str, rows: &[Row]) {
    let total: usize = rows.iter().map(|row| row.count).sum();
    println!("\n{title}: {} symbols, {total} in total", rows.len());
    println!(
        "{:>8} {:>9} {:>4}  {:<24} symbol",
        "count", "p", "bits", "code"
    );
    for row in rows {
        println!(
            "{:>8} {:>9.6} {:>4}  {:<24} {}",
            row.count,
            row.count as f64 / total as f64,
            row.bits,
            format!("{:0width$b}", row.code, width = row.bits as usize),
            row.symbol
        );
    }
    println!("average: {:.3} bits", average_bits(rows));
}

/// The code of a row in Rust, e.g. `0b0110, 4`.
fn rust_code(row: &Row) -> String {
    format!("0b{:0width$b}, {}", row.code, row.bits, width = row.bits as usize)
}

/// The Rust source of the tables.
fn source(statistics: &Statistics, tokens: &[Row], texts: &[Row]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The Huffman tables of shared links, generated by `cargo xtask share-tables` from
//! {} µcad files with {} tokens and {} characters, do not edit.
//!
//! A row is a symbol, its canonical Huffman code and the number of bits of the code.

/// The codes of the token kinds, with the text of kinds with a fixed text.
pub(super) const TOKENS: &[(&str, Option<&str>, u32, u8)] = &[",
        statistics.files,
        statistics.token_count(),
        statistics.char_count(),
    );
    let mut specials = String::new();
    for row in tokens {
        let code = rust_code(row);
        if row.symbol == OTHER_TOKEN {
            let _ = writeln!(
                specials,
                "\n/// The code of a token of a kind, which is not in [`TOKENS`], followed by its text.\npub(super) const TOKEN_OTHER: (u32, u8) = ({code});"
            );
            continue;
        }
        let fixed = match statistics.fixed.get(&row.symbol) {
            Some(text) => format!("Some({text:?})"),
            None => "None".into(),
        };
        let _ = writeln!(out, "    ({:?}, {fixed}, {code}),", row.symbol);
    }
    out.push_str("];\n\n/// The codes of the characters of the token texts.\n");
    out.push_str("pub(super) const CHARS: &[(char, u32, u8)] = &[\n");
    for row in texts {
        let code = rust_code(row);
        match row.symbol.as_str() {
            "End" => {
                let _ = writeln!(
                    specials,
                    "\n/// The code of the end of a token text.\npub(super) const TEXT_END: (u32, u8) = ({code});"
                );
            }
            "Other" => {
                let _ = writeln!(
                    specials,
                    "\n/// The code of a character, which is not in [`CHARS`], followed by its code point.\npub(super) const TEXT_OTHER: (u32, u8) = ({code});"
                );
            }
            symbol => {
                let _ = writeln!(out, "    ({symbol}, {code}),");
            }
        }
    }
    out.push_str("];\n");
    out.push_str(&specials);
    out
}

/// Lex the µcad files in `paths`, print the statistics and write the tables to `output`.
pub fn share_tables(paths: &[PathBuf], output: &Path) -> Result<(), Box<dyn Error>> {
    let mut statistics = Statistics::default();
    for file in mucad_files(paths) {
        statistics.add(&std::fs::read_to_string(&file)?);
    }
    if statistics.files == 0 {
        return Err("no µcad files found".into());
    }

    let tokens = rows(statistics.tokens.clone());
    let texts = rows(statistics.texts.iter().map(|(symbol, count)| {
        let symbol = match symbol {
            TextSymbol::Char(c) => format!("{c:?}"),
            TextSymbol::End => "End".into(),
            TextSymbol::Other => "Other".into(),
        };
        (symbol, *count)
    }));
    if let Some(row) = tokens.iter().chain(&texts).find(|row| row.bits > 32) {
        return Err(format!("the code of {} has more than 32 bits", row.symbol).into());
    }

    println!(
        "{} µcad files, {} tokens, {} characters",
        statistics.files,
        statistics.token_count(),
        statistics.char_count()
    );
    print_rows("Token kinds", &tokens);
    print_rows("Characters", &texts);

    std::fs::write(output, source(&statistics, &tokens, &texts))?;
    println!("\nWritten to {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes are a complete prefix code: no code is the start of another one.
    #[test]
    fn huffman_codes() {
        let codes = canonical_codes(&[5, 9, 12, 13, 16, 45]);
        assert_eq!(
            codes,
            [
                (0b1110, 4),
                (0b1111, 4),
                (0b100, 3),
                (0b101, 3),
                (0b110, 3),
                (0b0, 1)
            ]
        );
        let kraft: f64 = codes
            .iter()
            .map(|(_, bits)| 0.5f64.powi(*bits as i32))
            .sum();
        assert_eq!(kraft, 1.0);
        assert_eq!(canonical_codes(&[3]), [(0, 1)]);
    }

    /// Tokens are counted by their kind, characters of their texts with the ends of the texts.
    #[test]
    fn count_tokens() {
        let mut statistics = Statistics::default();
        statistics.add("fn f() {}");
        assert_eq!(statistics.tokens["Keyword(Fn)"], 1);
        assert_eq!(statistics.tokens["Identifier"], 1);
        assert_eq!(statistics.tokens["Whitespace"], 2);
        assert_eq!(statistics.tokens[OTHER_TOKEN], 1);
        assert_eq!(statistics.fixed["Keyword(Fn)"], "fn");
        assert_eq!(statistics.texts[&TextSymbol::Char(' ')], 2);
        assert_eq!(statistics.texts[&TextSymbol::Char('f')], 1);
        // The texts of the identifier and of the two whitespaces end.
        assert_eq!(statistics.texts[&TextSymbol::End], 3);
        assert_eq!(statistics.char_count(), 3);
    }
}
