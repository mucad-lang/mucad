# mucad VSCode plugin

Provides the syntax highlighting and LSP integration for mucad language plugin.

## Installation

### Manual installation

```sh
cd editors/vscode
npm install
npm run package
```

Open VS Code Command Palette (F1, Ctrl+Shift+P or Cmd+Shift+P) and select

```vscode
> Developer: Install Extension from Location…
```

Select the this folder `<root>/editors/vscode` to install the extension.
Future builds (e.g. after running `npm run package`) from this folder will be picked up by VS Code on extenstion reload.

### Build VSIX package

```sh
npm install
npm run vscode:package
```

This will produce a package `mucad-0.0.2.vsix`.

#### Install VSIX in VSCode

```sh
code --install-extension mucad-0.0.2.vsix
```

This will install the extension and you should see output similar to:

```
Extension `mucad-0.0.2.vsix` was successfully installed.
```

## ❤️ Support the project

This extension is part of the [mucad project](https://mucad.org).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/mucad/donate" target="_blank">
<img src="https://opencollective.com/mucad/donate/button@2x.png?color=blue" width=300 />
</a>
