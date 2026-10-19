#!/bin/sh
# Copyright © 2026 The µcad authors <info@mucad.org>
# SPDX-License-Identifier: AGPL-3.0-or-later

# Parse all µcad files of the examples and the standard library, which the µcad parser accepts,
# to find differences between the grammar and the µcad syntax.
set -e
cd "$(dirname "$0")"
root=../..
files=$(mktemp)
# `keypad/lib.mu` is unfinished, also the µcad parser rejects it.
find "$root/examples" "$root/crates/std/lib" -name '*.mu' | grep -v 'keypad/lib.mu' | sort > "$files"
# The highlight tests look up the grammar in the parser directories of a config.
config=$(mktemp)
printf '{"parser-directories": ["%s"]}' "$(cd .. && pwd)" > "$config"

tree-sitter generate
tree-sitter test --config-path "$config"
tree-sitter parse --config-path "$config" --quiet --stat --paths "$files"
