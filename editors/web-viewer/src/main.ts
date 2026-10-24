// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import init, { Mucad } from "mucad-wasm";
import { Dock } from "./dock";
import { addCompileShortcut, createEditor, setDiagnostics } from "./editor";
import { examples } from "./examples";
import { Viewer } from "./viewer";
import "./style.css";

/** The time after the last edit, before the code is compiled again. */
const COMPILE_DELAY_MS = 500;

const element = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const select = element<HTMLSelectElement>("examples");
const status = element<HTMLSpanElement>("status");
const exportButton = element<HTMLButtonElement>("export");
const exportButtonCaption = element<HTMLDivElement>("export-btn-caption");
const formatButton = element<HTMLButtonElement>("format");
const compileButton = element<HTMLButtonElement>("compile");
const autoUpdate = element<HTMLInputElement>("auto-update");
const diagnostics = element<HTMLPreElement>("diagnostics");

const dock = new Dock(element("dock"), [
  { id: "editor", title: "Code", element: element("editor") },
  { id: "view", title: "View", element: element("view") },
  { id: "diagnostics", title: "Diagnostics", element: diagnostics },
]);

await init();
const mucad = new Mucad();
const viewer = new Viewer(element("view"));

/** The model type of the last successful compilation, which decides the export format. */
let modelType: string | undefined;

/** The key of the auto-update setting in the local storage of the browser. */
const AUTO_UPDATE_KEY = "mucad.autoUpdate";
autoUpdate.checked = localStorage.getItem(AUTO_UPDATE_KEY) !== "false";

/** The pending compilation after an edit. */
let timer: number | undefined;

/** Mark that the code has changed since it was compiled, e.g. to compile it with the button. */
function setModified(modified: boolean) {
  compileButton.classList.toggle("btn-bright", modified);
}

/** Compile the code of the editor at once, instead of a pending compilation. */
function compileNow() {
  clearTimeout(timer);
  compile(editor.getValue());
}

/** Compile `code` and show its model, diagnostics and status. */
function compile(code: string) {
  setModified(false);
  const result = mucad.compile(code);
  diagnostics.textContent = result.report;
  setDiagnostics(editor, result.diagnostics);
  diagnostics.classList.toggle("error", !result.success);
  modelType = result.success ? result.modelType : undefined;

  try {
    switch (modelType) {
      case "3d":
        viewer.show3d(mucad.renderMeshes());
        break;
      case "2d":
        viewer.show2d(mucad.renderSketches());
        break;
      default:
        viewer.clear();
    }
  } catch (error) {
    diagnostics.textContent += String(error);
    diagnostics.classList.add("error");
    modelType = undefined;
  }

  status.textContent = modelType ? `${modelType.toUpperCase()} model` : "Errors";
  status.classList.toggle("error", !modelType);
  exportButton.disabled = modelType !== "3d" && modelType !== "2d";
  exportButtonCaption.textContent = modelType === "2d" ? "Export SVG" : "Export STL";
}

function format() {
  const code = editor.getValue();
  const formatted = mucad.format(code);
  if (formatted.success && formatted.formatted) {
      editor.setValue(formatted.code);
  }
}

/** Download the model as STL (3D) or SVG (2D). */
function exportModel() {
  const [data, type, extension] =
    modelType === "2d"
      ? [mucad.exportSvg(), "image/svg+xml", "svg"]
      : [new Uint8Array(mucad.exportStl()), "model/stl", "stl"];
  const name = select.selectedOptions[0]?.textContent?.split(" ")[0] ?? "model";
  const link = document.createElement("a");
  link.href = URL.createObjectURL(new Blob([data], { type }));
  link.download = `${name}.${extension}`;
  link.click();
  URL.revokeObjectURL(link.href);
}

// Detect if the user is on macOS
const isMac = /macintosh|mac os x/i.test(navigator.userAgent);
const modifierSymbol = isMac ? '⌘' : 'Ctrl+';

// Update all shortcut displays dynamically
document.querySelectorAll('.shortcut-key').forEach((el) => {
    el.textContent = modifierSymbol + el.textContent;
});

document.addEventListener('keydown', (e) => {
    const isCtrlOrCmd = e.ctrlKey || e.metaKey;
    if (!isCtrlOrCmd) return;

    // Ctrl+E or Cmd+E -> Export
    if (e.key.toLowerCase() === 'e') {
      e.preventDefault();
      exportModel();
    }

    // Ctrl+Enter or Cmd+Enter -> Compile
    if (e.key === 'Enter') {
        e.preventDefault();
        compileNow();
    }

    // Ctrl+F or Cmd+F -> Format
    if (e.key.toLowerCase() === 'f') {
        e.preventDefault();
        format();
    }
});

// Compile after each change with auto-update, otherwise with the compile button.
const editor = createEditor(element("editor"), (code) => {
  clearTimeout(timer);
  if (autoUpdate.checked) {
    timer = window.setTimeout(() => compile(code), COMPILE_DELAY_MS);
  } else {
    setModified(true);
  }
});
addCompileShortcut(editor, compileNow);

// Set version tag
const version = Mucad.versionString();
const versionTag = document.getElementById('version-tag');

if (versionTag) {
    versionTag.textContent = "v" + version;
}

examples.forEach((example, index) => select.add(new Option(example.name, String(index))));
// An example is compiled at once, also without auto-update.
select.addEventListener("change", () => {
  editor.setValue(examples[Number(select.value)].code);
  compileNow();
});
exportButton.addEventListener("click", exportModel);
formatButton.addEventListener("click", format);
compileButton.addEventListener("click", compileNow);
autoUpdate.addEventListener("change", () => {
  localStorage.setItem(AUTO_UPDATE_KEY, String(autoUpdate.checked));
  // Changes made without auto-update are compiled, when it is switched on.
  if (autoUpdate.checked && compileButton.classList.contains("btn-bright")) {
    compileNow();
  }
});


// The first example or the one given by the URL, e.g. `?example=1`.
select.value = String(Number(new URLSearchParams(location.search).get("example")) || 0);
editor.setValue(examples[Number(select.value)]?.code ?? examples[0].code);
compileNow();
