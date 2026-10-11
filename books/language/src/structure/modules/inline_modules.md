# Inline modules

Inline modules are modules are defined with the `mod` keyword,
followed by a name and a body `{...}` as the following example shows:

[![test](.test/inline_mod.svg)](.test/inline_mod.log)

```mu,inline_mod
mod my_module {
    // e.g. define a constant
    pub const VALUE = 1;
}
```

The following statements can be used within an inline module:

<!-- markdownlint-disable no-inline-html -->
| Statement                                      | Purpose                                     | Example        |
| ---------------------------------------------- | ------------------------------------------- | -------------- |
| [const assignment](../../assignments/const.md) | naming constants                            | `const Y = 1;` |
| [pub assignment](../../assignments/pub.md)     | exporting constants                         | `pub Y = 1;`   |
| [function](../functions.md)                    | separate calculations                       | `fn f() { }`   |
| [workbench](../workbenches)                    | build or transform 2D sketches and 3D parts | `part P() { }` |
| [module](README.md)                            | modularization of complex code              | `mod m { }`    |
| [use](../use.md)                               | use elements from other modules             | `use m;`       |
| [comment](../../doc/comments.md)               | for documentation                           | `//! comment`  |
