; Copyright © 2026 The µcad authors <info@mucad.org>
; SPDX-License-Identifier: AGPL-3.0-or-later

; Highlights with the capture names of Zed, also used by the tree-sitter CLI.
; If several patterns capture a node, the last one wins.

; Identifiers
;------------

(identifier) @variable

; Constants are written in capital letters, e.g. `PI`.
((identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]*$"))

; Captures of an identifier itself win over those of its parents.
(type_name
  (identifier) @type)

(marker
  "@" @variable.special
  (identifier) @variable.special)

; Definitions
;------------

(function_definition
  name: (identifier) @function)

(workbench_definition
  name: (identifier) @type)

(module_definition
  name: (identifier) @type)

(assignment
  qualifier: (qualifier "const")
  target: (identifier) @constant)

(assignment
  qualifier: (qualifier "prop")
  target: (identifier) @property)

(parameter
  name: (identifier) @variable.parameter)

(tuple_type_element
  (identifier) @property)

; Calls and element access
;-------------------------

; The last part of the path, e.g. `Circle` of `std::geo2d::Circle`.
(call
  method: (qualified_name
    (identifier) @function .))

(method_call
  method: (qualified_name
    (identifier) @function .))

; Workbenches are written in CamelCase, e.g. `Circle`.
(call
  method: (qualified_name
    (identifier) @type .)
  (#match? @type "^[A-Z][a-z]"))

(argument
  (identifier) @property
  "=")

(tuple_element_access
  (identifier) @property)

(attribute_access
  "#" @attribute
  (identifier) @attribute)

; Attributes
;-----------

(attribute
  "#[" @attribute
  "]" @attribute)

(inner_attribute
  "#![" @attribute
  "]" @attribute)

(attribute_item
  (identifier) @attribute)

(attribute_item
  (assignment
    target: (identifier) @attribute))

(attribute_item
  (call
    method: (qualified_name
      (identifier) @attribute .)))

; Literals
;---------

(bool_literal) @boolean

(number_literal) @number

(unit) @number

(string) @string

(format_expression
  "{" @punctuation.special
  "}" @punctuation.special) @embedded

(format_spec) @string.special

; Comments
;---------

(comment) @comment

(doc_comment) @comment.doc

(inner_doc_comment) @comment.doc

; Operators and punctuation
;--------------------------

(bin_op) @operator

(unary_op) @operator

[
  "="
  ".."
  "->"
] @operator

[
  ";"
  "."
  ","
  ":"
  "::"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

; Keywords
;---------

[
  "mod"
  "fn"
  "if"
  "else"
  "use"
  "as"
  "return"
  "init"
  "super"
] @keyword

(workbench_kind) @keyword

(qualifier) @keyword

(visibility) @keyword

(reserved) @keyword

(use_declaration
  "*" @operator)
