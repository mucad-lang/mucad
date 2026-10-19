# µcad Zed extension

µcad support for the [Zed](https://zed.dev) editor:

- Syntax highlighting with the [tree-sitter grammar](../tree-sitter-mucad) of this repository
- Outline of modules, sketches, parts, operations, functions and constants
- Indentation and bracket matching
- With the µcad language server (`mucad-lsp`): errors and warnings, code completion and
  formatting

Files with the extensions `.mu`, `.µcad`, `.mcad` and `.ucad` are µcad files.

## Installation

### 1. Install the language server

```sh
cargo install --path crates/lsp
```

The extension starts `mucad-lsp` from the `PATH`. Another binary can be set in the Zed settings:

```json
{
  "lsp": {
    "mucad-lsp": {
      "binary": { "path": "/path/to/mucad-lsp", "arguments": ["--stdio"] }
    }
  }
}
```

### 2. Install the extension

The extension is not published yet, it is installed as a development extension.
This requires [Rust](https://rustup.rs/) installed with `rustup`, Zed compiles the extension
with it.

1. Open the command palette (`Ctrl+Shift+P`, macOS: `Cmd+Shift+P`) and run
   `zed: install dev extension`.
2. Select this folder (`editors/zed`).

Zed loads the grammar from the µcad repository at the commit `rev` in `extension.toml`,
which must have been pushed.

## Structure

| Path | Content |
|---|---|
| `extension.toml` | The extension with its grammar and language server |
| `src/mucad.rs` | Starts the language server |
| `languages/mucad/config.toml` | File extensions, comments, brackets |
| `languages/mucad/highlights.scm` | Link to the highlight query of the grammar |
| `languages/mucad/brackets.scm`, `indents.scm`, `outline.scm` | Zed specific queries |

## Development

After changing the grammar in [`editors/tree-sitter-mucad`](../tree-sitter-mucad):

1. Check and commit the grammar with its generated parser.
2. Push the commit and set it as `rev` of `[grammars.mucad]` in `extension.toml`.
3. Run `zed: rebuild dev extension`.

To test a grammar before pushing, set `repository` temporarily to the local repository,
e.g. `file:///home/me/mucad`, and `rev` to a local commit.

The queries are checked against the grammar with the tree-sitter CLI, e.g.:

```sh
cd editors/tree-sitter-mucad
npx tree-sitter query ../zed/languages/mucad/outline.scm ../../examples/csg_cube.mu
```
