# tree-sitter-mucad

A [tree-sitter](https://tree-sitter.github.io/) grammar for µcad, with the queries for syntax
highlighting and tags, e.g. for the [Zed extension](../zed).

The µcad parser in [`crates/lang-parse`](../../crates/lang-parse) defines the syntax.
The grammar follows it: after changing the syntax, update the grammar and run the check below.
The grammar was written by Robin Appelman.

## Structure

| Path | Content |
|---|---|
| `grammar.js` | The grammar |
| `src/` | The parser generated from the grammar, which editors compile, e.g. Zed |
| `queries/highlights.scm` | Syntax highlighting with the capture names of Zed |
| `queries/tags.scm` | Definitions and references, e.g. for an outline |
| `test/corpus/` | Syntax trees of µcad code |
| `test/highlight/`, `test/tags/` | Expected highlights and tags |

If several patterns of `highlights.scm` capture a node, the last one wins.
A capture of a node wins over a capture of its parent, e.g. an identifier of a type name.

## Check

Install the [tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/creating-parsers/1-getting-started.html),
e.g. with `npm install`, and run:

```sh
cd editors/tree-sitter-mucad
npm run check  # or ./check.sh with the tree-sitter CLI in the PATH
```

The check generates the parser into `src/`, runs the tests and parses all µcad files of the
examples and the standard library, which must parse without errors.
Commit the generated parser together with the grammar.

To update the expected syntax trees of `test/corpus` after a change of the grammar,
run `npx tree-sitter test --update` and review the changes.
