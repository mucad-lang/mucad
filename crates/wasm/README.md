# µcad for WebAssembly

The µcad toolchain compiled to WebAssembly: compile µcad code, render its model and export it,
e.g. in a browser. The standard library is embedded, there is no file system access.

## Build

Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and build the package for the web:

```sh
wasm-pack build crates/wasm --target web --release --out-dir pkg
```

The package in `crates/wasm/pkg` contains the WebAssembly binary (about 5 MB),
the JavaScript bindings and TypeScript declarations.
Use `--target bundler` for bundlers like webpack or `--target nodejs` for Node.js.

## Usage

```js
import init, { Mucad } from "./pkg/mucad_wasm.js";

await init(); // Load the WebAssembly binary.

const mucad = new Mucad();
const result = mucad.compile(`Circle(radius = 10mm).extrude(5mm);`);

if (!result.success) {
  console.error(result.report);
} else if (result.modelType === "3d") {
  for (const mesh of mucad.renderMeshes()) {
    // mesh.positions: Float32Array, mesh.indices: Uint32Array, mesh.color: Float32Array | undefined
  }
} else if (result.modelType === "2d") {
  for (const sketch of mucad.renderSketches()) {
    // sketch.lines.positions: Float32Array, sketch.color: Float32Array | undefined
  }
}
```

`compile` keeps the model of the last successful compilation, the render and export methods use it.
They throw a string with the reason, e.g. render errors, if there is no model or it cannot be
rendered.

### Compile

| Method | Returns |
|---|---|
| `compile(code)` | `CompileResult` with `success`, `diagnostics` (see below), `report` (the diagnostics as text) and `modelType` (`"2d"`, `"3d"`, `"mixed"` or `"none"`) |
| `setResolution(mm)` | Sets the linear render resolution, e.g. `0.1` for smoother circles (default `0.25`) |

### Colors

A model is rendered into one mesh or sketch for each color, e.g. of `#[color = "red"]`.
`color` is `r, g, b, a` from `0.0` to `1.0` (sRGB), or `undefined` for the default color,
which is up to the viewer.

### 3D models

| Method | Returns |
|---|---|
| `renderMeshes()` | `Mesh[]`, each with `positions` (`x, y, z` triples), `indices` (triangles as index triples) and `color` |
| `exportStl()` | Binary STL as `Uint8Array` |

A mesh can be displayed with three.js:

```js
const geometry = new THREE.BufferGeometry();
geometry.setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3));
geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1));
geometry.computeVertexNormals();

const material = new THREE.MeshStandardMaterial();
if (mesh.color) {
  const [r, g, b, a] = mesh.color;
  material.color.setRGB(r, g, b, THREE.SRGBColorSpace);
  material.transparent = a < 1;
  material.opacity = a;
}
scene.add(new THREE.Mesh(geometry, material));
```

### 2D models

| Method | Returns |
|---|---|
| `renderSketches()` | `Sketch[]`, each with `color` and its geometries in three forms, see below |
| `exportSvg()` | SVG as string, with the colors of the models |

| `Sketch` property | Content |
|---|---|
| `polygons` | `Polygons`: the polygons with holes, e.g. for a canvas or SVG |
| `lines` | `Lines` with `positions`: line segments as `x0, y0, x1, y1`, e.g. for a `THREE.LineSegments` |
| `triangles` | `Mesh` in the XY plane (`z = 0`), e.g. for a filled `THREE.Mesh` |

`Polygons` stores the points of all rings one after another in `coordinates` (`x, y` pairs).
`rings` and `polygons` contain start indices with an additional end index:
ring `i` consists of the points `rings[i]` to `rings[i + 1]`,
polygon `j` of the rings `polygons[j]` to `polygons[j + 1]`.
The first ring of a polygon is its outline, the others are its holes.

```js
const { coordinates, rings, polygons } = sketch.polygons;
for (let p = 0; p + 1 < polygons.length; p++) {
  for (let r = polygons[p]; r < polygons[p + 1]; r++) {
    const ring = coordinates.subarray(2 * rings[r], 2 * rings[r + 1]);
    // ring: x0, y0, x1, y1, ...
  }
}
```

### Export

An export can be downloaded with a `Blob`:

```js
const blob = new Blob([mucad.exportStl()], { type: "model/stl" });
const link = document.createElement("a");
link.href = URL.createObjectURL(blob);
link.download = "model.stl";
link.click();
```

## Scope

Only compiling, rendering and exporting are available yet.
The model tree and the symbol library may be exposed in the future, e.g. for a model browser.

## Tests

The interface is tested natively:

```sh
cargo test -p mucad-wasm
```
