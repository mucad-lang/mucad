// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import init, { Mucad } from "mucad-wasm";
import { Dock } from "./dock";
import {
  addCompileShortcut,
  addSaveShortcut,
  createEditor,
  replaceCode,
  setDiagnostics,
} from "./editor";
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
const saveButton = element<HTMLButtonElement>("save");
const formatButton = element<HTMLButtonElement>("format");
const compileButton = element<HTMLButtonElement>("compile");
const autoUpdate = element<HTMLInputElement>("auto-update");
const diagnostics = element<HTMLPreElement>("diagnostics");
const shareButton = element<HTMLButtonElement>("share");
const sharePanel = element<HTMLDivElement>("share-panel");
const shareError = element<HTMLParagraphElement>("share-error");
const copyLinkButton = element<HTMLButtonElement>("copy-link");
const qrCanvas = element<HTMLCanvasElement>("qr-canvas");
const qrDownloadButton = element<HTMLButtonElement>("qr-download");
const stripComments = element<HTMLInputElement>("strip-comments");
const menu = element<HTMLDivElement>("menu");
const menuButton = element<HTMLButtonElement>("menu-button");

const dock = new Dock(element("dock"), [
  { id: "editor", title: "Code", element: element("editor") },
  { id: "view", title: "View", element: element("view") },
  { id: "diagnostics", title: "Diagnostics", element: diagnostics },
], "view");

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
    replaceCode(editor, formatted.code);
  }
}

/** A document of the example selection: an example or the code of a shared link. */
interface OpenDocument {
  /** The name in the example selection. */
  name: string;
  /** The code when it was last saved. */
  saved: string;
  /** The code in the editor, also when another document is shown. */
  code: string;
}

/** The documents by their value in the example selection, which are kept for this session. */
const docs = new Map<string, OpenDocument>();

/** The value of the shown document in the example selection. */
let shownDocument: string | undefined;

/** Add the document `value` to the documents, if it is not there yet, and return it. */
function docOf(value: string): OpenDocument {
  let doc = docs.get(value);
  if (!doc) {
    const code = examples[Number(value)]?.code ?? "";
    doc = { name: examples[Number(value)]?.name ?? value, saved: code, code };
    docs.set(value, doc);
  }
  return doc;
}

/** Show the document `value` in the editor with its unsaved changes. */
function showDocument(value: string) {
  select.value = value;
  shownDocument = value;
  editor.setValue(docOf(value).code);
}

/** Show the modified flag of each document in the example selection, and of the shown one. */
function updateModified() {
  for (const option of select.options) {
    const doc = docs.get(option.value);
    if (doc) {
      option.textContent = doc.code === doc.saved ? doc.name : `${doc.name} ●`;
    }
  }
  const doc = shownDocument === undefined ? undefined : docs.get(shownDocument);
  saveButton.classList.toggle("btn-bright", !!doc && doc.code !== doc.saved);
}

/** Save the shown document, which formats and compiles it. */
function save() {
  format();
  if (shownDocument !== undefined) {
    const doc = docOf(shownDocument);
    doc.saved = doc.code = editor.getValue();
  }
  updateModified();
  compileNow();
}

/** The value of the example selection for the code of a shared link. */
const SHARED_CODE = "shared";

/**
 * The code of a shared link in the URL: in the fragment, e.g. `#wrVBRAAEQ2lyY2xlKHIgPSAxMG1tKTsKAw`, or as
 * query parameter `?code=`. Without a shared link, there is neither code nor error.
 */
function sharedLink(): { code?: string; error?: string } {
  const link =
    location.hash.length > 1
      ? location.hash
      : new URLSearchParams(location.search).has("code")
        ? location.search
        : undefined;
  if (link === undefined) return {};
  try {
    return { code: Mucad.decodeLink(link) };
  } catch (error) {
    return { error: String(error) };
  }
}

/** Show the code of a shared link formatted, compile it and look at the whole model. */
function showSharedCode(code: string) {
  if (!select.querySelector(`option[value="${SHARED_CODE}"]`)) {
    select.add(new Option("Shared code", SHARED_CODE), 0);
  }
  const formatted = mucad.format(code);
  code = formatted.success && formatted.formatted ? formatted.code : code;
  // Another shared link replaces the shared code, also its unsaved changes.
  docs.set(SHARED_CODE, { name: "Shared code", saved: code, code });
  showDocument(SHARED_CODE);
  compileNow();
  viewer.resetCamera();
}

/** Show an error, e.g. of a shared link, in front of the diagnostics of the shown code. */
function showError(error: string) {
  diagnostics.textContent = `${error}\n\n${diagnostics.textContent}`;
  diagnostics.classList.add("error");
}

/** Show the error of a shared link in front of the diagnostics of the shown code. */
function showLinkError(error: string) {
  showError(`Cannot open the shared link: ${error}`);
}

/** The key of the strip comments setting in the local storage of the browser. */
const STRIP_COMMENTS_KEY = "mucad.stripComments";
stripComments.checked = localStorage.getItem(STRIP_COMMENTS_KEY) === "true";

/** Show an error in the share menu, or no error. */
function showShareError(error?: string) {
  shareError.textContent = error ?? "";
  shareError.hidden = error === undefined;
}

/** The link to the web viewer with the code of the editor, `undefined` after an error. */
function shareLink(): string | undefined {
  try {
    return Mucad.encodeLink(editor.getValue(), stripComments.checked);
  } catch (error) {
    showShareError(`Cannot share the code: ${error}`);
    return undefined;
  }
}

/** Copy the link with the code to the clipboard. */
async function copyLink() {
  const link = shareLink();
  if (link === undefined) return;
  try {
    await navigator.clipboard.writeText(link);
    copyLinkButton.textContent = "Copied";
    setTimeout(() => (copyLinkButton.textContent = "Copy link"), 1500);
  } catch (error) {
    showShareError(`Cannot copy the link: ${error}`);
  }
}

/** The number of modules of the quiet zone around a QR code. */
const QR_QUIET_ZONE = 4;

/** Draw the QR code of the link with the code dark on white, returns if it is drawn. */
function drawQrCode(): boolean {
  const link = shareLink();
  if (link === undefined) return false;
  let modules: Uint8Array;
  try {
    modules = Mucad.qrCode(link);
  } catch (error) {
    showShareError(`Cannot create the QR code: ${error}`);
    return false;
  }
  const width = Math.sqrt(modules.length);
  const size = width + 2 * QR_QUIET_ZONE;
  // At least 512 pixels for a sharp PNG.
  const scale = Math.max(2, Math.ceil(512 / size));
  qrCanvas.width = qrCanvas.height = size * scale;
  const context = qrCanvas.getContext("2d")!;
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, qrCanvas.width, qrCanvas.height);
  context.fillStyle = "#000000";
  modules.forEach((dark, index) => {
    if (!dark) return;
    const x = (index % width) + QR_QUIET_ZONE;
    const y = Math.floor(index / width) + QR_QUIET_ZONE;
    context.fillRect(x * scale, y * scale, scale, scale);
  });
  return true;
}

/**
 * Update the share menu for the code of the editor: the QR code, or the error, e.g. of code,
 * which is too long for a link.
 */
function updateShareMenu() {
  showShareError();
  const shareable = drawQrCode();
  for (const element of [copyLinkButton, qrCanvas, qrDownloadButton]) {
    element.hidden = !shareable;
  }
}

/** Show or hide the share menu. */
function toggleShareMenu() {
  sharePanel.hidden = !sharePanel.hidden;
  if (!sharePanel.hidden) updateShareMenu();
}

/** Show or hide the menu of actions and links on phones. */
function showMenu(open: boolean) {
  menu.classList.toggle("open", open);
  menuButton.setAttribute("aria-expanded", String(open));
}

/** The name of the shown example or code, e.g. for downloads. */
function modelName(): string {
  return select.selectedOptions[0]?.textContent?.split(" ")[0] ?? "model";
}

/** Download `blob` as file `name`. */
function download(blob: Blob, name: string) {
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.click();
  URL.revokeObjectURL(link.href);
}

/** Download the QR code as PNG. */
function downloadQrCode() {
  qrCanvas.toBlob((blob) => {
    if (blob) download(blob, `${modelName()}-qr.png`);
  }, "image/png");
}

/** Download the model as STL (3D) or SVG (2D). */
function exportModel() {
  const [data, type, extension] =
    modelType === "2d"
      ? [mucad.exportSvg(), "image/svg+xml", "svg"]
      : [new Uint8Array(mucad.exportStl()), "model/stl", "stl"];
  download(new Blob([data], { type }), `${modelName()}.${extension}`);
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
      if (!exportButton.disabled) exportModel();
    }

    // Ctrl+S or Cmd+S -> Save
    if (e.key.toLowerCase() === 's') {
        e.preventDefault();
        save();
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
  if (shownDocument !== undefined) docOf(shownDocument).code = code;
  updateModified();
  clearTimeout(timer);
  if (autoUpdate.checked) {
    timer = window.setTimeout(() => compile(code), COMPILE_DELAY_MS);
  } else {
    setModified(true);
  }
  // An open share menu shows the link of the changed code.
  if (!sharePanel.hidden) updateShareMenu();
});
addCompileShortcut(editor, compileNow);
addSaveShortcut(editor, save);

// Set version tag
const version = Mucad.versionString();
const versionTag = document.getElementById('version-tag');

if (versionTag) {
    versionTag.textContent = "v" + version;
}

examples.forEach((example, index) => select.add(new Option(example.name, String(index))));
// An example is compiled at once, also without auto-update, and shown as a whole.
// Its unsaved changes are kept, when another example is shown.
select.addEventListener("change", () => {
  showDocument(select.value);
  compileNow();
  viewer.resetCamera();
});
exportButton.addEventListener("click", exportModel);
saveButton.addEventListener("click", save);
formatButton.addEventListener("click", format);
compileButton.addEventListener("click", compileNow);
shareButton.addEventListener("click", toggleShareMenu);
copyLinkButton.addEventListener("click", copyLink);
qrDownloadButton.addEventListener("click", downloadQrCode);
stripComments.addEventListener("change", () => {
  localStorage.setItem(STRIP_COMMENTS_KEY, String(stripComments.checked));
  // The shown QR code is of the link with the new setting.
  updateShareMenu();
});
menuButton.addEventListener("click", () => showMenu(!menu.classList.contains("open")));
// The menu is hidden after an action or link, but not after switching auto-update.
menu.addEventListener("click", (event) => {
  if ((event.target as Element).closest("button, a")) showMenu(false);
});
// The share menu and the menu are hidden by a click outside of them.
document.addEventListener("click", (event) => {
  const target = event.target as Node;
  if (!sharePanel.contains(target) && !shareButton.contains(target)) sharePanel.hidden = true;
  if (!menu.contains(target) && !menuButton.contains(target)) showMenu(false);
});
autoUpdate.addEventListener("change", () => {
  localStorage.setItem(AUTO_UPDATE_KEY, String(autoUpdate.checked));
  // Changes made without auto-update are compiled, when it is switched on.
  if (autoUpdate.checked && compileButton.classList.contains("btn-bright")) {
    compileNow();
  }
});

// The code of a shared link, otherwise the first example or the one given by the URL,
// e.g. `?example=1`.
const shared = sharedLink();
if (shared.code !== undefined) {
  showSharedCode(shared.code);
} else {
  const example = Number(new URLSearchParams(location.search).get("example")) || 0;
  showDocument(String(examples[example] ? example : 0));
  compileNow();
  if (shared.error) showLinkError(shared.error);
}

// A shared link opened while the viewer runs, e.g. pasted into the address bar.
window.addEventListener("hashchange", () => {
  const shared = sharedLink();
  if (shared.code !== undefined) showSharedCode(shared.code);
  else if (shared.error) showLinkError(shared.error);
});
