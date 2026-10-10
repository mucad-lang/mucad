// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

fn builtin_docgen(path: impl AsRef<std::path::Path>) {
    let builtins = microcad_builtin::BuiltinRegistry::new();
    microcad_docgen::BuiltinMdbookGenerator::new(&builtins).write(path)
}
