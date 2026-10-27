// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import init, { Mucad } from "mucad-wasm";
import { Dock } from "./dock";
import {
  addCompileShortcut,
  addCompletion,
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
const documentsElement = element<HTMLSpanElement>("documents");
const documentName = element<HTMLInputElement>("document-name");
const documentListButton = element<HTMLButtonElement>("document-list-button");
const documentList = element<HTMLDivElement>("document-list");
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

/** A document: an example, the code of a shared link or an own document of the user. */
interface OpenDocument {
  /** Whether it is an example, shared code or an own document, which can be renamed. */
  kind: "example" | "shared" | "own";
  /** The name in the list of documents. */
  name: string;
  /** The name in the preamble of a shared link, e.g. of the example. */
  sharedName?: string;
  /** The code when it was last saved. */
  saved: string;
  /** The code in the editor, also when another document is shown. */
  code: string;
}

/**
 * The documents by their keys, which are kept for this session: the index of an example,
 * `SHARED_CODE` or `own-1`, `own-2`… of own documents.
 */
const docs = new Map<string, OpenDocument>();

/** The key of the shown document. */
let shownDocument: string | undefined;

/** The number of own documents, which have been created, for their keys. */
let ownDocuments = 0;

/** Add the example `key` to the documents, if it is not there yet, and return the document. */
function docOf(key: string): OpenDocument {
  let doc = docs.get(key);
  if (!doc) {
    const code = examples[Number(key)]?.code ?? "";
    const name = examples[Number(key)]?.name ?? key;
    doc = { kind: "example", name, sharedName: name, saved: code, code };
    docs.set(key, doc);
  }
  return doc;
}

/** Show the document `key` in the editor with its unsaved changes. */
function showDocument(key: string) {
  shownDocument = key;
  documentName.value = docOf(key).name;
  editor.setValue(docOf(key).code);
}

/** Show the document `key` from the list, compile it at once and look at the whole model. */
function openDocument(key: string) {
  showDocument(key);
  compileNow();
  viewer.resetCamera();
}

/**
 * Apply the name in the name field to the shown document: an own document is renamed, an
 * example or shared code is copied into a new own document with the name and the unsaved
 * changes, the example keeps its saved code.
 *
 * The name is made unique, e.g. `gear (2)`, an empty name is not applied.
 */
function renameDocument() {
  if (shownDocument === undefined) return;
  const doc = docOf(shownDocument);
  const name = documentName.value.trim();
  if (!name || name === doc.name) {
    documentName.value = doc.name;
    return;
  }
  const names = new Set([...docs.values()].filter((other) => other !== doc).map((d) => d.name));
  let unique = name;
  for (let number = 2; names.has(unique); number++) unique = `${name} (${number})`;

  if (doc.kind === "own") {
    doc.name = doc.sharedName = unique;
  } else {
    const key = `own-${++ownDocuments}`;
    const code = editor.getValue();
    docs.set(key, { kind: "own", name: unique, sharedName: unique, saved: doc.saved, code });
    doc.code = doc.saved;
    shownDocument = key;
  }
  documentName.value = unique;
  updateModified();
}

/** Show the documents in groups: own documents, shared code and examples. */
function renderDocumentList() {
  const keys = (kind: OpenDocument["kind"]) =>
    [...docs].filter(([, doc]) => doc.kind === kind).map(([key]) => key);
  const groups: [string, string[]][] = [
    ["Own documents", keys("own")],
    ["Shared code", keys("shared")],
    ["Examples", examples.map((_, index) => String(index))],
  ];
  documentList.replaceChildren();
  for (const [title, group] of groups) {
    if (!group.length) continue;
    const heading = document.createElement("p");
    heading.className = "document-group";
    heading.textContent = title;
    documentList.append(heading);
    for (const key of group) {
      const doc = docOf(key);
      const button = document.createElement("button");
      button.setAttribute("role", "option");
      button.setAttribute("aria-selected", String(key === shownDocument));
      button.textContent = doc.code === doc.saved ? doc.name : `${doc.name} ●`;
      button.addEventListener("click", () => {
        showDocumentList(false);
        openDocument(key);
      });
      documentList.append(button);
    }
  }
}

/** Show or hide the list of documents. */
function showDocumentList(open: boolean) {
  if (open) renderDocumentList();
  documentList.hidden = !open;
  documentListButton.setAttribute("aria-expanded", String(open));
}

/** Show the modified flag of the shown document, and of each one in an open list. */
function updateModified() {
  const doc = shownDocument === undefined ? undefined : docs.get(shownDocument);
  const modified = !!doc && doc.code !== doc.saved;
  saveButton.classList.toggle("btn-bright", modified);
  documentsElement.classList.toggle("modified", modified);
  if (!documentList.hidden) renderDocumentList();
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

/** The key of the code of a shared link. */
const SHARED_CODE = "shared";

/**
 * The code and the name in the preamble of a shared link in the URL: in the fragment, e.g.
 * `#wrUBQUQABENpcmNsZShyID0gMTBtbSk7CgM`, or as query parameter `?code=`. Without a shared
 * link, there is neither code nor error.
 */
function sharedLink(): { code?: string; name?: string; error?: string } {
  const link =
    location.hash.length > 1
      ? location.hash
      : new URLSearchParams(location.search).has("code")
        ? location.search
        : undefined;
  if (link === undefined) return {};
  try {
    const shared = Mucad.decodeLinkWithPreamble(link);
    const { code, preamble } = shared;
    shared.free();
    const { name } = JSON.parse(preamble);
    return { code, name: typeof name === "string" ? name : undefined };
  } catch (error) {
    return { error: String(error) };
  }
}

/**
 * Show the code of a shared link formatted under its `name`, e.g. `brick.mu (shared)`, compile
 * it and look at the whole model.
 */
function showSharedCode(code: string, name?: string) {
  const formatted = mucad.format(code);
  code = formatted.success && formatted.formatted ? formatted.code : code;
  // Another shared link replaces the shared code, also its unsaved changes.
  const label = name === undefined ? "Shared code" : `${name} (shared)`;
  docs.set(SHARED_CODE, { kind: "shared", name: label, sharedName: name, saved: code, code });
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

/**
 * The link to the web viewer with the code of the editor and the name of the shown document in
 * its preamble, `undefined` after an error.
 */
function shareLink(): string | undefined {
  const name = shownDocument === undefined ? undefined : docOf(shownDocument).sharedName;
  const preamble = JSON.stringify(name === undefined ? {} : { name });
  try {
    return Mucad.encodeLinkWithPreamble(editor.getValue(), preamble, stripComments.checked);
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

/** The name of the shown document, e.g. for downloads. */
function modelName(): string {
  return (shownDocument === undefined ? undefined : docs.get(shownDocument)?.sharedName) ?? "model";
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
// The completions are copied, so that the objects of WebAssembly can be freed at once.
addCompletion((code, lineNo, column) =>
  mucad.complete(code, lineNo, column).map((completion) => {
    const { name, kind, doc } = completion;
    completion.free();
    return { name, kind, doc };
  }),
);

// Set version tag
const version = Mucad.versionString();
const versionTag = document.getElementById('version-tag');

if (versionTag) {
    versionTag.textContent = "v" + version;
}

// A document is compiled at once, also without auto-update, and shown as a whole.
// Its unsaved changes are kept, when another document is shown.
documentListButton.addEventListener("click", () => showDocumentList(documentList.hidden));
// A new name is applied with Enter or by leaving the name field, Escape restores the name.
documentName.addEventListener("change", renameDocument);
documentName.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && shownDocument !== undefined) {
    documentName.value = docOf(shownDocument).name;
  }
  if (event.key === "Enter" || event.key === "Escape") documentName.blur();
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
// The list of documents, the share menu and the menu are hidden by a click outside of them.
document.addEventListener("click", (event) => {
  const target = event.target as Node;
  if (!documentList.contains(target) && !documentListButton.contains(target)) {
    showDocumentList(false);
  }
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
  showSharedCode(shared.code, shared.name);
} else {
  const example = Number(new URLSearchParams(location.search).get("example")) || 0;
  showDocument(String(examples[example] ? example : 0));
  compileNow();
  if (shared.error) showLinkError(shared.error);
}

// A shared link opened while the viewer runs, e.g. pasted into the address bar.
window.addEventListener("hashchange", () => {
  const shared = sharedLink();
  if (shared.code !== undefined) showSharedCode(shared.code, shared.name);
  else if (shared.error) showLinkError(shared.error);
});
