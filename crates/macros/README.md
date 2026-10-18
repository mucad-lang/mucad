# µcad Built-in Proc-Macros

[![Crates.io](https://img.shields.io/crates/v/mucad-builtin-proc-macros.svg)](https://crates.io/crates/mucad-builtin-proc-macros)
[![Documentation](https://docs.rs/mucad-builtin-proc-macros/badge.svg)](https://docs.rs/mucad-builtin-proc-macros/)

This crate provides procedural macros used to simplify the creation of built-in symbols for the µcad language.

## Macros

The following derive macros are provided:

| Macro                | Description                                                                                                                    > |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `BuiltinPrimitive2D` | Derives the necessary traits for a 2D primitive. The struct is expected to have fields that define the primitive's parameters. |
| `BuiltinPrimitive3D` | Derives the necessary traits for a 3D primitive. The struct is expected to have fields that define the primitive's parameters. |
| `BuiltinOperation`   | Derives the necessary traits for a general operation that can be applied to primitives.                                        |
| `BuiltinOperation2D` | Derives the necessary traits for an operation that is specific to 2D primitives.                                               |
| `BuiltinOperation3D` | Derives the necessary traits for an operation that is specific to 3D primitives.                                               |

## Usage

These macros are intended to be used within the `mucad-builtin` crate to define new built-in symbols.

Here is an example of how to define a new 2D primitive:

```rust,ignore
use mucad_builtin_proc_macros::BuiltinPrimitive2D;
use mucad_lang::value::{Number, Integer};

#[derive(BuiltinPrimitive2D)]
struct MyCircle {
    radius: Integer,
}
```

This will generate the necessary boilerplate to make `MyCircle` available as a built-in function in µcad, which can be called like `__builtin::geo2d::MyCircle(radius=5)`.

The macro automatically generates:

- An `id()` for the symbol.
- A `help()` string from the doc comments.
- The `output_type()` of the symbol.
- The `kind()` of the symbol (primitive or operation).
- A `workpiece_function()` to create a new instance of the symbol.
- A list of `parameters()` for the symbol.
