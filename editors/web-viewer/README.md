# µcad web viewer

Edit, display and export µcad models in a browser, with the µcad toolchain compiled to
WebAssembly (`crates/wasm`).

- [Monaco](https://microsoft.github.io/monaco-editor/) editor with µcad syntax highlighting
- [three.js](https://threejs.org/) view of 3D models and 2D sketches
- Examples from the repository's `examples` folder
- Models in their colors, e.g. of `#[color = "red"]`
- Errors and warnings as squiggles in the code
- Export of 3D models as STL and of 2D sketches as SVG
- Shared links with the code, e.g. `https://play.mucad.org/#tUAGABlAmvhY1ptjUgs`: the *Share*
  menu copies a link with the code of the editor and shows its QR code with a PNG download,
  *Strip comments* shares the code without comments
- Dockable panels with [Dockview](https://dockview.dev/)

The code is compiled again shortly after each edit, errors and warnings are shown in the
diagnostics panel.

## Build

Requirements: [Rust](https://rustup.rs/) with the target `wasm32-unknown-unknown`,
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and [Node.js](https://nodejs.org/).

```sh
cd editors/web-viewer
npm run wasm     # Build the WebAssembly package into crates/wasm/pkg
npm install
npm run dev      # Development server, e.g. http://localhost:5173
```

`npm run build` type checks and builds a static site into `dist`, which can be served from any
directory, e.g. with `npm run preview`.
After changes in the Rust code, run `npm run wasm` again.

An example can be opened directly with its index in the URL, e.g. `?example=1`.

## Examples

The examples are listed in `src/examples.ts` and imported from the `examples` folder.
