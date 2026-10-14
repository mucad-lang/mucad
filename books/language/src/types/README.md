# Types

The µcad type system consists of a group of builtin types.
The type system is *static*, which means a every value has a fixed type
which cannot be changed or [overwritten](../assignments/value.md#no-shadowing).

Here is a complete list of the builtin types:

| Type                               | Description                               | Type Declarations                                                  | Example Values                                                         |
| ---------------------------------- | ----------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| [*Boolean*](primitives.md#bool)    | Boolean value                             | `Bool`                                                             | `true`, `false`                                                        |
| [*Integer*](primitives.md#integer) | Integer value without unit                | `Integer`                                                          | `4`, `-1`                                                              |
| [*Quantity*](quantities/)          | Floating point value with or without unit | `Scalar`, `Length`, `Area`, `Volume`, `Density`, `Angle`, `Weight` | `0.5`, `50%`, `-1.23e10`, `-4mm`, `1.3m2`, `2cm²`, `23.0e5deg`, `100g` |
| [*String*](primitives.md#string)   | UTF-8 text string                         | `String`                                                           | `"Hello, World!"`                                                      |
| [*Array*](collections/array/)      | List of values with *common type*         | `[Integer]`                                                        | `[1,2,3]`, `[1m,2cm,3µm]`                                              |
| [*Tuple*](collections/tuples/)     | List of named values or distinct types    | `(Length,Scalar,Bool)`, `(x:Scalar,y:Length)`, `(x:Scalar,Length)` | `(4mm,4.0,true)`, `(x=4.0,y=4mm)`, `(x=4.0,4mm)`                       |
| [*Model*](models/)                 | Geometric 2D or 3D model                  | `Model`                                                            | `std::geo3d::Cube(2mm)`                                                |

## Declaration

The examples in the table above declare the type explicitly.
However, we can use units to declare the implicitly.
Using *units* is recommended and what you get in return is that declarations
are quite handy:

[![test](.test/types_def_vs_decl.svg)](.test/types_def_vs_decl.log)

```mu,types_def_vs_decl
x: Length = 4mm; // explicit type declaration
y = 4mm; // implicit type declaration via units.
```

Declarations without any initializations are *not allowed* in µcad.
Hence, the following example will fail:

[![test](.test/types_no_declaration.svg)](.test/types_no_declaration.log)

```mu,types_no_declaration?mode=todo
x: Length;         // parse_error
```

However, for [parameter lists](../flow/calls/args_params.md#parameters) in
[functions](../structure/functions/) and [workbenches](../structure/workbenches/elements/),
you can declare the type only but also pass a default value:

[![test](.test/types_bundles_functions.svg)](.test/types_bundles_functions.log)

```mu,types_bundles_functions
fn f(x = 4mm) {} // use unit (with default)
fn g(x: Length) {} // use type (without default)
```

> [!NOTE]
> Find out more about what types are used for in the sections about
> [*argument matching*](../flow/argument_match/) and [*assignments*](../assignments/).
