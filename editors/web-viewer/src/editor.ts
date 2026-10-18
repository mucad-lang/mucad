// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as monaco from "monaco-editor/editor/editor.api";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";
import { theme } from "./theme";

self.MonacoEnvironment = { getWorker: () => new EditorWorker() };

const LANGUAGE = "mucad";

/** Register the µcad language with keywords, types, units and comments. */
function registerLanguage() {
  monaco.languages.register({ id: LANGUAGE, extensions: [".mu"] });
  monaco.languages.setMonarchTokensProvider(LANGUAGE, {
    keywords: [
      "as", "assembly", "const", "else", "enum", "extern", "fn", "if", "init", "match",
      "material", "mod", "op", "part", "prop", "pub", "return", "sketch", "struct", "super",
      "type", "unit", "use",
    ],
    types: [
      "Integer", "Scalar", "String", "Color", "Length", "Area", "Volume", "Angle", "Weight",
      "Density", "Bool",
    ],
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
export function createEditor(element: HTMLElement, onChange: (code: string) => void) {
  registerLanguage();
  defineTheme();
  const editor = monaco.editor.create(element, {
    language: LANGUAGE,
    theme: LANGUAGE,
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    tabSize: 4,
  });
  editor.onDidChangeModelContent(() => onChange(editor.getValue()));
  return editor;
}
