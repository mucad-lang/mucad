# Source Files

*Source files* are files that contain µcad source code.
Such files must either have the extension `.µcad`, `.mcad` or `.ucad`.

A source file can include the following types of *statements* which we will all
discuss within this book:

| Statement                                   | Purpose                                     | Example        |
| ------------------------------------------- | ------------------------------------------- | -------------- |
| [expression](../expressions/README.md#)     | calculate values                            | `x * 5;`       |
| [assignment](../assignments/values.md)      | store values                                | `y = x;`       |
| [const assignment](../assignments/const.md) | naming constants                            | `const Y = 1;` |
| [pub assignment](../assignments/pub.md)     | exporting constants                         | `pub Y = 1;`   |
| [function](functions.md)                    | separate calculations                       | `fn f() { }`   |
| [workbench](workbenches/)                   | build or transform 2D sketches and 3D parts | `part P() { }` |
| [module](modules/)                          | modularization of complex code              | `mod m { }`    |
| [if](../flow/conditions.md)                 | process conditions                          | `if x > 1 { }` |
| [use](use.md)                               | use symbols from other modules              | `use m;`       |
| [call](../flow/calls)                       | use functions and workbenches               | `f();`         |
| [comment](../doc/comments.md)               | for documentation                           | `// comment`   |

In its simplest form, a µcad program consists of a single file containing one
or more of the above statements.

A source file can serve as both a module and a workbench.
You can use it to provide structure (for example, by organizing submodules) or
as a kind of workbench where you add statements to generate outputs—such as a
2D graphic or a 3D model.

The workbench section of the file is only evaluated if it is in the main file
(the one that `microcad` was called with).

The following examples illustrate this: a circle and a sphere are created, each
with a radius of one centimeter.

[![test](.test/source_file_2D.svg)](.test/source_file_2D.log)

```mu,source_file_2D
// simply draw a circle
std::geo2d::Circle(radius = 1cm);
```

[![test](.test/source_file_3D.svg)](.test/source_file_3D.log)

```mu,source_file_3D
// simply draw a sphere
std::geo3d::Sphere(radius = 1cm);
```

Statements within a source file can represent either a 2D or a 3D model — but
not both at the same time.
Mixing 2D and 3D statements in the same file will result in an error:

[![test](.test/source_file_mixed.svg)](.test/source_file_mixed.log)

```mu,source_file_mixed?result=fail
std::geo2d::Circle(radius = 1cm);
std::geo3d::Sphere(radius = 1cm); // error: can't mix 2D and 3D
```

However, µcad programs can also be split across multiple files.
To include other files, the [`mod`](modules/external_modules.md)
statement is used...
