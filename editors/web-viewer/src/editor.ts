// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as monaco from "monaco-editor/editor/editor.api";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";
// The API contains no features of the editor, e.g. the suggestions of completions.
import "monaco-editor/editor/contrib/suggest/browser/suggestController";
import { type Diagnostic, Severity } from "mucad-wasm";
import { theme } from "./theme";

self.MonacoEnvironment = { getWorker: () => new EditorWorker() };

const LANGUAGE = "mucad";

/** The keywords of µcad. */
const KEYWORDS = [
  "as", "assembly", "const", "else", "enum", "extern", "fn", "if", "init", "match",
  "material", "mod", "op", "part", "prop", "pub", "return", "sketch", "struct", "super",
  "type", "unit", "use",
];

/** The built-in types of µcad. */
const TYPES = [
  "Integer", "Scalar", "String", "Color", "Length", "Area", "Volume", "Angle", "Weight",
  "Density", "Bool",
];

/** Register the µcad language with keywords, types, units and comments. */
function registerLanguage() {
  monaco.languages.register({ id: LANGUAGE, extensions: [".mu"] });
  monaco.languages.setMonarchTokensProvider(LANGUAGE, {
    keywords: KEYWORDS,
    types: TYPES,
    tokenizer: {
      root: [
        [/\/\/.*$/, "comment"],
        [/\/\*/, "comment", "@comment"],
        [/"([^"\\]|\\.)*"/, "string"],
        [/#\[[^\]]*\]/, "annotation"],
        [/@\w+/, "annotation"],
        [/\d+(\.\d+)?(%|[a-zA-Zµ°²³]+)?/, "number"],
        [/\b(true|false)\b/, "number"],
        [/[a-zA-Z_]\w*/, { cases: { "@keywords": "keyword", "@types": "type", "@default": "identifier" } }],
      ],
      comment: [
        [/\*\//, "comment", "@pop"],
        [/./, "comment"],
      ],
    },
  });
  monaco.languages.setLanguageConfiguration(LANGUAGE, {
    comments: { lineComment: "//", blockComment: ["/*", "*/"] },
    brackets: [["{", "}"], ["[", "]"], ["(", ")"]],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
  });
}

/** Define the editor theme from the µcad theme. */
function defineTheme() {
  const color = (hex: string) => hex.slice(1);
  monaco.editor.defineTheme(LANGUAGE, {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: color(theme.guide), fontStyle: "bold" },
      { token: "type", foreground: color(theme.signal) },
      { token: "number", foreground: color(theme.signal) },
      { token: "string", foreground: color(theme.brighter) },
      { token: "comment", foreground: color(theme.darker), fontStyle: "italic" },
      { token: "annotation", foreground: color(theme.brighter) },
      { token: "identifier", foreground: color(theme.bright) },
    ],
    colors: {
      "editor.background": theme.primary,
      "editor.foreground": theme.bright,
      "editorLineNumber.foreground": theme.darker,
      "editorCursor.foreground": theme.signal,
      "editor.selectionBackground": theme.darker,
    },
  });
}

/** Create a µcad code editor in `element`, `onChange` is called after each edit. */
/** The owner of the markers of the compiler. */
const MARKER_OWNER = "mucad";

/** Show diagnostics of the compiler as markers in the code, e.g. squiggles below errors. */
export function setDiagnostics(
  editor: monaco.editor.IStandaloneCodeEditor,
  diagnostics: Diagnostic[],
) {
  const model = editor.getModel();
  if (!model) return;
  monaco.editor.setModelMarkers(model, MARKER_OWNER, diagnostics.map(marker));
}

/** The Monaco marker of a diagnostic. */
function marker(diagnostic: Diagnostic): monaco.editor.IMarkerData {
  return {
    message: diagnostic.message,
    severity: {
      [Severity.Error]: monaco.MarkerSeverity.Error,
      [Severity.Warning]: monaco.MarkerSeverity.Warning,
      [Severity.Advice]: monaco.MarkerSeverity.Hint,
    }[diagnostic.severity],
    startLineNumber: diagnostic.startLineNo,
    startColumn: diagnostic.startColumn,
    endLineNumber: diagnostic.endLineNo,
    endColumn: diagnostic.endColumn,
    source: "µcad",
  };
}

export function createEditor(element: HTMLElement, onChange: (code: string) => void) {
  registerLanguage();
  defineTheme();
  const editor = monaco.editor.create(element, {
    language: LANGUAGE,
    theme: LANGUAGE,
    automaticLayout: true,
    minimap: { enabled: false },
    // Suggestions and their documentation may overlap other panels, outside of the dock,
    // which clips its panels. The class gives them the colors of the theme.
    fixedOverflowWidgets: true,
    overflowWidgetsDomNode: Object.assign(document.body.appendChild(document.createElement("div")), {
      id: "editor-widgets",
      className: "monaco-editor",
    }),
    fontSize: 14,
    tabSize: 4,
  });
  editor.onDidChangeModelContent(() => onChange(editor.getValue()));
  return editor;
}

/**
 * Run `compile` with Ctrl+Enter (Cmd+Enter on macOS), also when the editor has the focus,
 * where Monaco would insert a line otherwise.
 */
export function addCompileShortcut(
  editor: monaco.editor.IStandaloneCodeEditor,
  compile: () => void,
) {
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, compile);
}

/**
 * Run `save` with Ctrl+S (Cmd+S on macOS), also when the editor has the focus, instead of
 * saving the page by the browser.
 */
export function addSaveShortcut(editor: monaco.editor.IStandaloneCodeEditor, save: () => void) {
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, save);
}

/**
 * Replace the code of the editor as one edit, which can be undone, unlike `setValue`.
 * The cursor stays at its line and column, as far as they still exist.
 */
export function replaceCode(editor: monaco.editor.IStandaloneCodeEditor, code: string) {
  const model = editor.getModel();
  if (!model) return;
  const position = editor.getPosition();
  editor.pushUndoStop();
  editor.executeEdits("format", [{ range: model.getFullModelRange(), text: code }]);
  editor.pushUndoStop();
  if (position) editor.setPosition(model.validatePosition(position));
}

/** A symbol to complete a path with, e.g. `Circle` for `std::geo2d::Ci`. */
export interface Completion {
  name: string;
  /** e.g. `sketch`, `fn` or `mod`. */
  kind: string;
  /** The documentation in Markdown. */
  doc?: string;
}

/** The Monaco kinds of the kinds of completions. */
const COMPLETION_KINDS: Record<string, monaco.languages.CompletionItemKind> = {
  library: monaco.languages.CompletionItemKind.Module,
  mod: monaco.languages.CompletionItemKind.Module,
  sketch: monaco.languages.CompletionItemKind.Class,
  part: monaco.languages.CompletionItemKind.Class,
  op: monaco.languages.CompletionItemKind.Method,
  fn: monaco.languages.CompletionItemKind.Function,
  "built-in": monaco.languages.CompletionItemKind.Function,
  const: monaco.languages.CompletionItemKind.Constant,
};

/**
 * Complete names with the symbols of `complete` at a line and column (from 1, in UTF-16 code
 * units), e.g. after `::`, and with keywords and types, unless the name is after `::`.
 */
export function addCompletion(
  complete: (code: string, lineNo: number, column: number) => Completion[],
) {
  monaco.languages.registerCompletionItemProvider(LANGUAGE, {
    // Paths are completed after `::`, like by the language server.
    triggerCharacters: [":"],
    provideCompletionItems(model, position) {
      const word = model.getWordUntilPosition(position);
      const range = new monaco.Range(
        position.lineNumber,
        word.startColumn,
        position.lineNumber,
        word.endColumn,
      );
      const symbols = complete(model.getValue(), position.lineNumber, position.column);
      const suggestions: monaco.languages.CompletionItem[] = symbols.map((symbol) => ({
        label: symbol.name,
        kind: COMPLETION_KINDS[symbol.kind] ?? monaco.languages.CompletionItemKind.Variable,
        detail: symbol.kind,
        documentation: symbol.doc ? { value: symbol.doc } : undefined,
        insertText: symbol.name,
        range,
      }));
      const before = model.getLineContent(position.lineNumber).slice(0, word.startColumn - 1);
      if (!before.endsWith("::")) {
        const words = (names: string[], kind: monaco.languages.CompletionItemKind) =>
          names.map((name) => ({ label: name, kind, insertText: name, range }));
        suggestions.push(
          ...words(KEYWORDS, monaco.languages.CompletionItemKind.Keyword),
          ...words(TYPES, monaco.languages.CompletionItemKind.TypeParameter),
        );
      }
      return { suggestions };
    },
  });
}
