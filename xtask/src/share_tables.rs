// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate the Huffman tables of shared links from the symbols of µcad files, see
//! `mucad_driver::share`.

use std::collections::{BTreeMap, BinaryHeap};
use std::error::Error;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use mucad_driver::prelude as mu;

use mu::share::SharedSymbol;

/// Directories, which are not searched for µcad files.
const EXCLUDED_DIRS: &[&str] = &["target", "node_modules", "dist", "pkg", ".git"];

/// Identifiers in at least this number of files are in the dictionary.
const MIN_DICTIONARY_FILES: usize = 2;

/// The whitespace of formatted code.
const WHITESPACES: [&str; 3] = [" ", "\n", "\n\n"];

/// The symbol of escape rows, e.g. of identifiers, which are not in the dictionary.
const ESCAPE: &str = "Escape";

/// The number of occurrences of the symbols of µcad files.
#[derive(Default)]
pub struct Statistics {
    /// The number of files.
    pub files: usize,
    /// The number of each whitespace.
    whitespaces: BTreeMap<&'static str, usize>,
    /// The number of structural tokens of each kind, e.g. `Keyword(Fn)`.
    structurals: BTreeMap<String, usize>,
    /// The number of each identifier and the number of files with the identifier.
    identifiers: BTreeMap<String, (usize, usize)>,
    /// The identifiers of each file in the order of their first occurrence.
    file_identifiers: Vec<Vec<String>>,
    /// The number of each character of fragments.
    fragment_chars: BTreeMap<char, usize>,
}

impl Statistics {
    /// Count the symbols of the µcad `code`.
    pub fn add(&mut self, code: &str) {
        self.files += 1;
        let mut identifiers: Vec<String> = Vec::new();
        for symbol in mu::share::shared_symbols(code) {
            match symbol {
                SharedSymbol::Whitespace(text) => *self.whitespaces.entry(text).or_default() += 1,
                SharedSymbol::Structural { kind, .. } => {
                    *self.structurals.entry(kind).or_default() += 1
                }
                SharedSymbol::Identifier(name) => {
                    let (count, files) = self.identifiers.entry(name.clone()).or_default();
                    *count += 1;
                    if !identifiers.contains(&name) {
                        *files += 1;
                        identifiers.push(name);
                    }
                }
                SharedSymbol::Fragment(text) => {
                    for c in text.chars() {
                        *self.fragment_chars.entry(c).or_default() += 1;
                    }
                }
            }
        }
        self.file_identifiers.push(identifiers);
    }

    /// If an identifier is in the dictionary.
    fn in_dictionary(&self, name: &str) -> bool {
        self.identifiers
            .get(name)
            .is_some_and(|(_, files)| *files >= MIN_DICTIONARY_FILES)
    }

    /// The tables: whitespace, structural tokens, identifiers, characters of identifiers and
    /// of fragments. The symbols are Rust literals, except for structural tokens and escapes.
    fn tables(&self) -> [Vec<Row>; 5] {
        // Each whitespace and structural token is in the table.
        let whitespaces = WHITESPACES.iter().map(|text| {
            let count = self.whitespaces.get(text).copied().unwrap_or_default();
            (format!("{text:?}"), count.max(1))
        });
        let structurals = mu::share::structural_tokens().map(|(kind, _)| {
            let count = self.structurals.get(&kind).copied().unwrap_or_default();
            (kind, count.max(1))
        });

        let mut locals = 0;
        let mut identifiers = Vec::new();
        for (name, (count, _)) in &self.identifiers {
            match self.in_dictionary(name) {
                true => identifiers.push((format!("{name:?}"), *count)),
                false => locals += count,
            }
        }
        identifiers.push((ESCAPE.into(), locals.max(1)));

        // The identifiers, which are not in the dictionary, are spelled once in each file.
        let mut identifier_chars: BTreeMap<char, usize> = BTreeMap::new();
        for name in self.file_identifiers.iter().flatten() {
            if !self.in_dictionary(name) {
                for c in name.chars() {
                    *identifier_chars.entry(c).or_default() += 1;
                }
            }
        }
        let chars = |chars: &BTreeMap<char, usize>| {
            let mut rows: Vec<_> = (chars.iter())
                .map(|(c, count)| (format!("{c:?}"), *count))
                .collect();
            rows.push((ESCAPE.into(), 1));
            rows
        };

        [
            rows(whitespaces),
            rows(structurals),
            rows(identifiers),
            rows(chars(&identifier_chars)),
            rows(chars(&self.fragment_chars)),
        ]
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
    format!(
        "0b{:0width$b}, {}",
        row.code,
        row.bits,
        width = row.bits as usize
    )
}

/// Write the rows of a table as Rust constant `name`, the escape row as constant `escape`.
fn write_table(
    out: &mut String,
    doc: &str,
    name: &str,
    kind: &str,
    rows: &[Row],
    escape: Option<(&str, &str)>,
) {
    let _ = writeln!(
        out,
        "\n/// {doc}\npub(super) const {name}: &[({kind}, u32, u8)] = &["
    );
    let mut escape_row = None;
    for row in rows {
        match row.symbol == ESCAPE {
            true => escape_row = Some(row),
            false => {
                let _ = writeln!(out, "    ({}, {}),", row.symbol, rust_code(row));
            }
        }
    }
    out.push_str("];\n");
    if let (Some((name, doc)), Some(row)) = (escape, escape_row) {
        let _ = writeln!(
            out,
            "\n/// {doc}\npub(super) const {name}: (u32, u8) = ({});",
            rust_code(row)
        );
    }
}

/// The Rust source of the tables.
fn source(statistics: &Statistics, tables: &[Vec<Row>; 5]) -> String {
    let [
        whitespaces,
        structurals,
        identifiers,
        identifier_chars,
        fragment_chars,
    ] = tables;
    let mut out = format!(
        "// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The Huffman tables of shared links, generated by `cargo xtask share-tables` from
//! {} µcad files, do not edit.
//!
//! A row is a symbol, its canonical Huffman code and the number of bits of the code.
",
        statistics.files
    );
    write_table(
        &mut out,
        "The codes of whitespace.",
        "WHITESPACES",
        "&str",
        whitespaces,
        None,
    );

    let texts: BTreeMap<_, _> = mu::share::structural_tokens().collect();
    let _ = writeln!(
        out,
        "\n/// The codes of the structural tokens by their kinds, with their texts.\npub(super) const STRUCTURALS: &[(&str, &str, u32, u8)] = &["
    );
    for row in structurals {
        let _ = writeln!(
            out,
            "    ({:?}, {:?}, {}),",
            row.symbol,
            texts[&row.symbol],
            rust_code(row)
        );
    }
    out.push_str("];\n");

    write_table(
        &mut out,
        "The codes of the identifiers of the dictionary.",
        "IDENTIFIERS",
        "&str",
        identifiers,
        Some((
            "IDENTIFIER_LOCAL",
            "The code of an identifier of the code, which is not in [`IDENTIFIERS`].",
        )),
    );
    write_table(
        &mut out,
        "The codes of the characters of identifiers, which are not in the dictionary.",
        "IDENTIFIER_CHARS",
        "char",
        identifier_chars,
        Some((
            "IDENTIFIER_CHAR_OTHER",
            "The code of a character, which is not in [`IDENTIFIER_CHARS`], followed by its code point.",
        )),
    );
    write_table(
        &mut out,
        "The codes of the characters of fragments.",
        "FRAGMENT_CHARS",
        "char",
        fragment_chars,
        Some((
            "FRAGMENT_CHAR_OTHER",
            "The code of a character, which is not in [`FRAGMENT_CHARS`], followed by its code point.",
        )),
    );
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

    let tables = statistics.tables();
    if let Some(row) = tables.iter().flatten().find(|row| row.bits > 32) {
        return Err(format!("the code of {} has more than 32 bits", row.symbol).into());
    }

    println!("{} µcad files", statistics.files);
    let titles = [
        "Whitespace",
        "Structural tokens",
        "Identifiers",
        "Characters of identifiers",
        "Characters of fragments",
    ];
    for (title, rows) in titles.iter().zip(&tables) {
        print_rows(title, rows);
    }

    std::fs::write(output, source(&statistics, &tables))?;
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

    /// Identifiers in several files are in the dictionary, the others are spelled once in
    /// each file.
    #[test]
    fn count_symbols() {
        let mut statistics = Statistics::default();
        statistics.add("x = a + a;");
        statistics.add("y = a + 1mm;");
        assert_eq!(statistics.whitespaces[" "], 8);
        assert_eq!(statistics.structurals["OperatorAssignment"], 2);
        assert_eq!(statistics.identifiers["a"], (3, 2));
        assert!(statistics.in_dictionary("a"));
        assert!(!statistics.in_dictionary("x"));
        // A unit is an identifier.
        assert_eq!(statistics.fragment_chars[&'1'], 1);
        assert_eq!(statistics.identifiers["mm"], (1, 1));

        let [_, _, identifiers, identifier_chars, _] = statistics.tables();
        let count = |rows: &[Row], symbol: &str| {
            rows.iter()
                .find(|row| row.symbol == symbol)
                .map(|row| row.count)
        };
        assert_eq!(count(&identifiers, "\"a\""), Some(3));
        // `x`, `y` and `mm` are escaped.
        assert_eq!(count(&identifiers, ESCAPE), Some(3));
        assert_eq!(count(&identifier_chars, "'x'"), Some(1));
    }
}
