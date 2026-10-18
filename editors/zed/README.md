# µcad zed plugin

This extension provides support for the µcad in the Zed editor, including syntax highlighting, code folding, and intelligent language server features (LSP) such as formatting and displaying errors.

## Installation

### 1. Install the `mucad-lsp`

```sh
cargo install --path crates/lsp mucad-lsp
```

### 2. Install into Zed

1. Open Zed.
2. Open the Command Palette:
    - macOS: `Cmd + Shift + P`
    - Windows/Linux: `Ctrl + Shift + P`

3. Type zed: extensions and press `Enter` to open the Extensions view.
4. Click on **"Install Dev Extension"** in the top right corner (or select it from the command palette).
5. Navigate to this very folder and click **Open**.
