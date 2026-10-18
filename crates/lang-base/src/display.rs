// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub trait DisplayOneLine: std::fmt::Display {
    fn fmt_one_line(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_one_line())
    }

    // Re-implement this function if you want a custom one-liner.
    fn to_string_one_line(&self) -> String {
        let full = self.to_string();
        let line = full.lines().next().unwrap_or("");
        shorten(line, 80)
    }
}

/// Shortens given string to it's first line and to `max_chars` characters.
pub fn shorten(s: &str, max_chars: usize) -> String {
    let s = s.trim_end();
    if s.chars().count() > max_chars {
        format!(
            "{}…",
            s.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        s.to_string()
    }
}
