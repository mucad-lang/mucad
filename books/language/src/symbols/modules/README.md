# Modules

A module is a way to organize and group code into logical units.
Module are declared with the `mod` keyword.
Modules help to manage scope and define namespaces in your code.
Each module has a unique name that must be written as [`lower_snake_case`](./../../appendix/coding.md).

In µcad, there are two kinds of modules:

* [*inline modules*](inline_modules.md): Inline modules defined by a name and a body inside a file: `mod my_module { ... }`.
* [*file modules*](file_modules.md): File modules have their definition in a separate file: `mod my_file;`.

The visibility of modules of modules can be controlled via the `pub` qualifier.
If a module is qualified as `pub`, it becomes visible to external.

## Example

The following example shows two nested *inline modules* carrying a public constant and
a public function which then are used from outside the modules:

[![test](.test/mod_example.svg)](.test/mod_example.log)

```mu,mod_example
// define private module
mod my {
    // define public module
    pub mod math {
        // define PI as a public constant value
        pub const PI = 3.14159;

        // define some public calculation function
        pub fn abs(x: Scalar) -> Scalar {
            if x < 0 {
                return -x;
            } else {
                return x;
            }
        }
    }
}

// use the function and the constant
x = my::math::abs(-1.0) * my::math::PI;
```
