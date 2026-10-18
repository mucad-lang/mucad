// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { defineConfig } from "vite";

export default defineConfig({
  // Relative paths, so the build can be served from any directory.
  base: "./",
  server: {
    // The examples are loaded from the repository's `examples` folder.
    fs: { allow: ["../.."] },
  },
  // The WebAssembly package loads its binary relative to its JavaScript module.
  optimizeDeps: { exclude: ["mucad-wasm"] },
});
