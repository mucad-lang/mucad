# Functions

Functions provide a way to encapsulate frequently used code into sub-routines.
These sub-routines can then be [called](../../flow/calls/function_calls.md) to
execute their code with a specific set of
[parameters](../../flow/calls/parameters.md).

The function definition starts with the keyword `fn`, followed by an
*identifier*, a *parameter list*, and a *function body*.

[![test](.test/example.svg)](.test/example.log)

```mu,example
// define function print_error with text as parameter of type String
fn print_error(text: String) {
    // code body
    std::print("ERROR: {text}");
}

print_error("first");
print_error("second");
```

Output
  :
   ```txt
   ERROR: first
   ERROR: second
   ```

## Default Parameters

Parameters can include *default values*.
Default values are specified without an explicit type annotation, but their
type is inferred from the unit of the provided value.

[![test](.test/function_default.svg)](.test/function_default.log)

```mu,function_default
fn f(x: Scalar, y = 1mm) -> Length {
    x * y
}

std::debug::assert_eq([f(2), 2mm]);
std::debug::assert_eq([f(2, 2mm), 4mm]);
```

Functions may be declared within [source files](../source_file.md),
[modules](../modules/) or [workbenches](../workbenches/).
