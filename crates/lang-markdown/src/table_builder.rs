// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Block;

/// Column text alignment for Markdown tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl Alignment {
    fn format_cell(&self, text: &str, width: usize) -> String {
        match self {
            Alignment::Left => format!("{:<width$}", text, width = width),
            Alignment::Right => format!("{:>width$}", text, width = width),
            Alignment::Center => format!("{:^width$}", text, width = width),
        }
    }

    fn to_delimiter(self, width: usize) -> String {
        let width = width.max(3);
        match self {
            Alignment::Left => format!(":{:-<width$}", "", width = width - 1),
            Alignment::Right => format!("{:-<width$}:", "", width = width - 1),
            Alignment::Center => format!(":{:-<width$}:", "", width = width - 2),
        }
    }
}

/// Defines a table column with a header name and alignment rule.
#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub align: Alignment,
}

impl Column {
    pub fn new(title: impl Into<String>, align: Alignment) -> Self {
        Self {
            title: title.into(),
            align,
        }
    }

    pub fn left(title: impl Into<String>) -> Self {
        Self::new(title, Alignment::Left)
    }

    pub fn center(title: impl Into<String>) -> Self {
        Self::new(title, Alignment::Center)
    }

    pub fn right(title: impl Into<String>) -> Self {
        Self::new(title, Alignment::Right)
    }
}

impl<S: Into<String>> From<S> for Column {
    fn from(title: S) -> Self {
        Self::left(title)
    }
}

/// A builder for constructing formatted Markdown tables.
#[derive(Debug, Clone, Default)]
pub struct TableBuilder {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Set table columns.
    pub fn columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Column>,
    {
        self.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Add a single row to the table.
    pub fn row<I, S>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let formatted_row = row
            .into_iter()
            .map(|cell| cell.into().replace('|', "\\|").replace('\n', " "))
            .collect();
        self.rows.push(formatted_row);
        self
    }

    /// Add multiple rows.
    pub fn rows<I, R, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for r in rows {
            self = self.row(r);
        }
        self
    }

    fn col_widths(&self) -> Vec<usize> {
        let num_cols = self.columns.len();
        let mut widths = vec![3; num_cols];

        for (i, col) in self.columns.iter().enumerate() {
            widths[i] = widths[i].max(col.title.len());
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate().take(num_cols) {
                widths[i] = widths[i].max(cell.len());
            }
        }

        widths
    }
}

impl std::fmt::Display for TableBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.columns.is_empty() {
            return Ok(());
        }

        let widths = self.col_widths();

        // 1. Header row
        write!(f, "|")?;
        self.columns.iter().enumerate().try_for_each(|(i, col)| {
            write!(
                f,
                " {} |",
                Alignment::Left.format_cell(&col.title, widths[i])
            )
        })?;
        writeln!(f)?;

        // 2. Delimiter row
        write!(f, "|")?;
        self.columns
            .iter()
            .enumerate()
            .try_for_each(|(i, col)| write!(f, " {} |", col.align.to_delimiter(widths[i])))?;
        writeln!(f)?;

        // 3. Data rows
        for row in &self.rows {
            write!(f, "|")?;
            for i in 0..self.columns.len() {
                let cell = row.get(i).map(String::as_str).unwrap_or("");
                let align = self.columns[i].align;
                write!(
                    f,
                    " {} |",
                    align.format_cell(cell, widths.get(i).copied().unwrap_or(0))
                )?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl From<TableBuilder> for Block {
    fn from(table: TableBuilder) -> Self {
        Block::Table(table.to_string().lines().map(String::from).collect())
    }
}
