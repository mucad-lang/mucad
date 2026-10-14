# Inline Identifiers

In some cases, the parameter name is already included in the argument expression.
If there is only one (or multiple identical) identifier(s) within an expression
and it matches a parameter of the same type, that parameter will be automatically
matched.

[![test](.test/argument_match_auto.svg)](.test/argument_match_auto.log)

```mu,argument_match_auto
fn f(x: Integer, y: Integer) -> Integer { x * y }
x = 1;
f(x, y = 2); // matches because argument `x` matches the name of parameter `x`
```

Even when using a more complex expression a unique identifier can match an argument:

[![test](.test/argument_match_single_identifier.svg)](.test/argument_match_single_identifier.log)

```mu,argument_match_single_identifier
fn f(x: Integer, y: Integer) -> Integer { x * y }
x = 1;
y = 2;
f(x * 2, y * y); // matches because `x` and `y` match parameter names `x` and `y`
```

[![test](.test/argument_match_auto_err.svg)](.test/argument_match_auto_err.log)

```mu,argument_match_auto_err?result=fail
fn f(x: Integer, y: Integer) -> Integer { x * y }
x = 1;
y = 2;
f(x * y, y * x); // error: cannot be matched because arguments are not unique.
```
