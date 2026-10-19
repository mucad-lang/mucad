; Copyright © 2026 The µcad authors <info@mucad.org>
; SPDX-License-Identifier: AGPL-3.0-or-later

; Definitions in the outline panel, e.g. `sketch Disc` or `fn area`.

(module_definition
  (doc_comment)* @annotation
  "mod" @context
  name: (identifier) @name) @item

(workbench_definition
  (doc_comment)* @annotation
  kind: (workbench_kind) @context
  name: (identifier) @name) @item

(init_definition
  (doc_comment)* @annotation
  "init" @name
  parameters: (parameter_list) @context) @item

(function_definition
  (doc_comment)* @annotation
  "fn" @context
  name: (identifier) @name) @item

(assignment_statement
  (doc_comment)* @annotation
  (assignment
    qualifier: (qualifier) @context
    target: (identifier) @name)) @item
