# Format Strings

*Format strings* are [strings](primitives.md#string) which include a special
notation to insert expressions into a text.

[![test](.test/format_string.svg)](.test/format_string.log)

```mu,format_string
std::debug::assert_eq(["{2 + 5}", "7"]);
```

Usually they are used to insert parameters into text:

[![test](.test/format_string_value.svg)](.test/format_string_value.log)

```mu,format_string_value
fn print_length(length: Length) {
    std::print("{length}");
}

print_length(7mm);
```

## Bad Expression in Format String

If a format string expression cannot be evaluated, you will get an error.

[![test](.test/format_string_err.svg)](.test/format_string_err.log)

```mu,format_string_err#fail
fn print_length(length: Length) {
    std::print("{size}"); // error: size is not known
}

print_length(7mm);
```
