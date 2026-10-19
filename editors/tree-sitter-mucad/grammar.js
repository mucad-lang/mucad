/**
 * @file Modern programming language for CAD
 * @author Robin Appelman <robin@icewind.nl>
 * @license AGPL-3.0-or-later
 *
 * Maintained with the µcad parser in `crates/lang-parse`, which defines the syntax.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const ALPHA_LOWER = /[a-z]/;
const DIGITS = /[0-9]+/;
const INT = /0|[1-9][0-9]*/;
const EXPONENT = /[eE][+-]?[0-9]+/;

module.exports = grammar({
  name: "mucad",

  extras: ($) => [/\s|\\\r?\n/, $.comment],

  supertypes: ($) => [$.primary, $.statement, $.literal],

  word: ($) => $.identifier,

  reserved: {
    global: ($) => [
      "mod",
      "part",
      "sketch",
      "op",
      "fn",
      "if",
      "else",
      "use",
      "as",
      "return",
      "pub",
      "const",
      "prop",
      "init",
      "__plugin",
      "assembly",
      "material",
      "unit",
      "enum",
      "struct",
      "match",
      "type",
      "super",
      "extern",
    ],
  },

  rules: {
    source_file: ($) => optional($.statement_list),

    comment: ($) => choice($.single_line_comment, $.multi_line_comment),
    doc_comment: ($) => /\/\/\/[^\r\n]*/,
    inner_doc_comment: ($) => /\/\/![^\r\n]*/,
    single_line_comment: ($) => /\/\/([^\/!][^\r\n]*)?/,
    multi_line_comment: ($) =>
      token(seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")),
    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_']*/,

    float: ($) =>
      token(
        prec(
          1,
          seq(
            optional("-"),
            // No `1.` without decimals, which would take the `1.` of a range `[1..4]`.
            choice(
              seq(choice(seq(INT, ".", DIGITS), seq(".", DIGITS)), optional(EXPONENT)),
              seq(INT, EXPONENT),
            ),
          ),
        ),
      ),
    int: ($) => token(INT),
    number_literal: ($) =>
      choice(
        seq($.float, optional($.unit)),
        seq(token(prec(1, seq(optional("-"), INT))), optional($.unit)),
      ),

    string: ($) =>
      seq(
        '"',
        optional($.string_literal_inner),
        repeat(seq($.format_expression, optional($.string_literal_inner))),
        '"',
      ),
    string_literal_inner: ($) =>
      /(\\(\\|"|b|f|n|r|t|u\d{4})|[^"{}\\]||\{\{|}})*/,

    format_expression: ($) =>
      seq(
        "{",
        field("expression", $.expression),
        field("format", optional(seq(":", $.format_spec))),
        "}",
      ),
    format_spec: ($) =>
      choice(
        seq("0", field("width", $.int), ".", field("accuracy", $.int)),
        seq(".", field("accuracy", $.int)),
        seq("0", field("width", $.int)),
      ),
    bool_literal: ($) => choice("true", "false"),
    unit: ($) =>
      token.immediate(
        choice(
          seq(
            optional("µ"),
            repeat1(ALPHA_LOWER),
            optional(choice("²", "³", "2", "3")),
            optional(
              seq("/", repeat1(ALPHA_LOWER), optional(choice("²", "³", "2", "3"))),
            ),
          ),
          "°",
          "%",
          "'",
        ),
      ),
    literal: ($) => choice($.number_literal, $.bool_literal),

    _path_part: ($) => choice($.identifier, "super"),
    qualified_name: ($) =>
      seq(optional("::"), $._path_part, repeat(seq("::", $._path_part))),

    type: ($) => choice($.array_type, $.tuple_type, $.type_name),
    // E.g. `Length`, `Integer` or `Color`.
    type_name: ($) => $.identifier,

    type_annotation: ($) => seq(":", $.type),

    array_type: ($) => seq("[", $.type, "]"),

    tuple_type: ($) =>
      seq(
        "(",
        $.tuple_type_element,
        repeat(seq(",", $.tuple_type_element)),
        optional(","),
        ")",
      ),
    tuple_type_element: ($) => choice(seq($.identifier, ":", $.type), $.type),

    range_expression: ($) => seq($.expression, "..", $.expression),

    array_expression: ($) =>
      seq(
        "[",
        choice($.range_expression, $.list_expression),
        "]",
        optional($.unit),
      ),

    tuple_expression: ($) => seq("(", $.argument_list, ")"),

    tuple_element_access: ($) => seq(".", $.identifier),
    array_element_access: ($) => seq("[", $.expression, "]"),
    attribute_access: ($) => seq("#", $.identifier),
    element_access: ($) =>
      choice(
        $.method_call,
        $.array_element_access,
        $.tuple_element_access,
        $.attribute_access,
      ),

    visibility: ($) => "pub",
    use_statement: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        optional($.visibility),
        "use",
        $.use_declaration,
        ";",
      ),
    use_declaration: ($) =>
      seq(
        optional("::"),
        $._path_part,
        repeat(seq("::", choice($._path_part, "*"))),
        optional(seq("as", $.identifier)),
      ),
    expression: ($) =>
      seq(
        repeat($.unary_op),
        $.primary,
        repeat($.element_access),
        repeat(
          seq(
            $.bin_op,
            repeat($.unary_op),
            $.primary,
            repeat($.element_access),
          ),
        ),
      ),
    bin_op: ($) =>
      choice(
        "+",
        "-",
        "*",
        "/",
        "|",
        "&",
        "^",
        ">",
        "<",
        ">=",
        "<=",
        "==",
        "~",
        "!=",
        "and",
        "or",
        "xor",
      ),
    bracket_expression: ($) => seq("(", $.expression, ")"),
    unary_op: ($) => choice("-", "+", "!"),
    primary: ($) =>
      choice(
        $.literal,
        $.marker,
        $.block,
        $.call,
        $.qualified_name,
        $.string,
        $.array_expression,
        $.bracket_expression,
        $.tuple_expression,
        $.if_statement,
      ),
    list_expression: ($) =>
      seq($.expression, repeat(seq(",", $.expression)), optional(",")),
    expression_statement: ($) =>
      seq(optional($.attribute_list), $.expression, ";"),
    // A block needs no `;`, unless it is part of an expression, e.g. `{ ... }.translate(x = 1mm);`.
    block_statement: ($) => prec(-1, seq(optional($.attribute_list), $.block)),
    final_expression_statement: ($) =>
      seq(optional($.attribute_list), $.expression),

    argument: ($) =>
      prec(1, choice(seq($.identifier, "=", $.expression), $.expression)),
    argument_list: ($) =>
      seq($.argument, repeat(seq(",", $.argument)), optional(",")),
    _call_op: ($) => seq("(", optional($.argument_list), ")"),
    method_call: ($) =>
      seq(
        ".",
        field("method", $.qualified_name),
        field("arguments", $._call_op),
      ),
    call: ($) =>
      seq(field("method", $.qualified_name), field("arguments", $._call_op)),

    attribute_item: ($) => choice($.assignment, $.call, $.identifier),
    attribute: ($) =>
      seq("#[", $.attribute_item, repeat(seq(",", $.attribute_item)), "]"),
    inner_attribute: ($) =>
      seq("#![", $.attribute_item, repeat(seq(",", $.attribute_item)), "]"),
    attribute_list: ($) => repeat1($.attribute),

    qualifier: ($) => choice("const", "prop"),

    assignment: ($) =>
      seq(
        field("visibility", optional($.visibility)),
        field("qualifier", optional($.qualifier)),
        field("target", $.identifier),
        field("type", optional($.type_annotation)),
        "=",
        field("value", $.expression),
      ),
    assignment_statement: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        field("attributes", optional($.attribute_list)),
        $.assignment,
        ";",
      ),

    marker: ($) => seq("@", $.identifier),

    parameter: ($) =>
      choice(
        seq(
          field("name", $.identifier),
          field("type", optional($.type_annotation)),
          "=",
          field("default", $.expression),
        ),
        seq(field("name", $.identifier), field("type", $.type_annotation)),
      ),

    _parameters: ($) =>
      seq($.parameter, repeat(seq(",", $.parameter)), optional(",")),
    parameter_list: ($) => seq("(", optional($._parameters), ")"),

    function_definition: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        field("attributes", optional($.attribute_list)),
        field("visibility", optional($.visibility)),
        "fn",
        field("name", $.identifier),
        field("parameters", $.parameter_list),
        optional(seq("->", field("return", $.type))),
        field("body", $.block),
      ),

    return_statement: ($) => seq("return", optional($.expression), ";"),
    if_statement: ($) =>
      seq(
        "if",
        $.expression,
        $.block,
        optional(seq("else", choice($.block, $.if_statement))),
      ),
    workbench_kind: ($) => choice("sketch", "part", "op"),
    workbench_definition: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        field("attributes", optional($.attribute_list)),
        field("visibility", optional($.visibility)),
        field("kind", $.workbench_kind),
        field("name", $.identifier),
        // A workbench with initializers only needs no building plan.
        field("parameters", optional($.parameter_list)),
        field("body", $.block),
      ),

    block: ($) => seq("{", optional($.statement_list), "}"),

    init_definition: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        field("attributes", optional($.attribute_list)),
        "init",
        field("parameters", $.parameter_list),
        field("body", $.block),
      ),

    module_definition: ($) =>
      seq(
        field("doc", repeat($.doc_comment)),
        field("attributes", optional($.attribute_list)),
        field("visibility", optional($.visibility)),
        "mod",
        field("name", $.identifier),
        choice(field("body", $.block), ";"),
      ),

    statement: ($) =>
      prec(
        1,
        choice(
          $.workbench_definition,
          $.module_definition,
          $.function_definition,
          $.init_definition,
          $.use_statement,
          $.return_statement,
          $.if_statement,
          $.block_statement,
          $.inner_attribute,
          $.assignment_statement,
          $.expression_statement,
          $.reserved,
          $.inner_doc_comment,
        ),
      ),

    statement_list: ($) =>
      choice(
        seq(repeat1($.statement), optional($.final_expression_statement)),
        $.final_expression_statement,
      ),

    reserved: ($) =>
      choice(
        "__plugin",
        "assembly",
        "material",
        "unit",
        "enum",
        "struct",
        "match",
        "type",
        "extern",
      ),
  },
});
