# Match Priorities

A single parameter can match an argument in several ways, each with a defined *priority*.
These priorities become  important when calling [workbenches](../calls/workbench_calls.md)
which support overloaded initialization.

| Priority<br>⭣ high to low | Matches                                                       | Example Parameters                  | Example Arguments |
| ------------------------- | ------------------------------------------------------------- | ----------------------------------- | ----------------- |
| Empty List                | Empty arguments with empty parameters                         | `()`                                | `()`              |
| Identifier                | Match argument identifier with parameter identifier           | `(x: Scalar)`                       | `(x=1)`           |
| Shortened Identifier      | Match argument identifier with shortened parameter identifier | <nobr>`(max_height: Scalar)`</nobr> | `(m_h=1)`         |
| Type                      | Match argument type with parameter type                       | `(x: Length)`                       | `(1mm)`           |
| Compatible Type           | Match argument type with compatible parameter type            | `(x: Scalar)`                       | `f(1)`            |
| Default                   | Match parameter defaults                                      | `(x=1mm)`                           | `()`              |

The match strategy is to try all priorities in order from highest (`Empty`) to
lowest (`Default`) until all arguments match a parameter.

## Match Empty List

Matches when both the arguments and parameters are empty.

[![test](.test/argument_match_empty.svg)](.test/argument_match_empty.log)

```mu,argument_match_empty
fn f() {} // no parameters

f(); // no arguments
```

## Match Identifier

The following example demonstrates calling a function `f` with each argument specified by name:

[![test](.test/argument_match_id.svg)](.test/argument_match_id.log)

```mu,argument_match_id
fn f(width: Length, height: Length) -> Area { width * height }

x = f(height = 2cm, width = 1cm); // call f() with parameters in arbitrary order

std::debug::assert_eq([x, 2cm²]);
```

## Match Short Identifier

A parameter can also be matched using it's short identifier.

The short form consists of the first letter of each word separated by underscores (_).

[![test](.test/argument_match_short.svg)](.test/argument_match_short.log)

```mu,argument_match_short
fn f(width: Length, height: Length) -> Area { width * height }

// use short identifiers
std::debug::assert_eq([f(w = 1cm, h = 2cm), 2cm²]);
// can be mixed
std::debug::assert_eq([f(w = 1cm, height = 2cm), 2cm²]);
```

Here are some usual examples of short identifiers:

| Identifier                    | Short Identifier |
| ----------------------------- | ---------------- |
| `parameter`                   | `p`              |
| `my_parameter`                | `m_p`            |
| `my_very_long_parameter_name` | `m_v_l_p_n`      |
| `my_Parameter`                | `m_P`            |
| `MyParameter`                 | `M`              |
| `myParameter`                 | `m`              |

## Match Type

Nameless values can be used if all parameter types of the called function
(or workbench) are distinct.

[![test](.test/argument_match_type.svg)](.test/argument_match_type.log)

```mu,argument_match_type
fn f(a: Scalar, b: Length, c: Area) {} // warning: unused a,b,c
// Who needs names?
f(1.0, 2cm, 3cm²);
```

## Match Compatible Type

Nameless arguments can also be compatible with *parameter types*, even if they
are not identical.

[![test](.test/argument_match_type_compatible.svg)](.test/argument_match_type_compatible.log)

```mu,argument_match_type_compatible
fn f(a: Scalar, b: Length, c: Area) {} // warning: unused a,b,c
// giving an integer `1` to a `Scalar` parameter `a`
f(1, 2cm, 3cm²);
```

## Match Default

If an argument is not provided and its parameter has a default value defined,
the default will be used.

[![test](.test/argument_match_default.svg)](.test/argument_match_default.log)

```mu,argument_match_default
fn f(a = 1mm) {} // warning: unused a
// a has default
f();
```

## Mix'em all

You can combine all these methods.

[![test](.test/argument_match_mix.svg)](.test/argument_match_mix.log)

```mu,argument_match_mix
fn f(a: Scalar, b: Length, c = 2cm, d: Length) -> Volume {} // warning: unused a,b,c,d

// `a` gets the Integer (1) which is compatible to Scalar (1.0)
// `b` is named
// `c` gets it's default
// `d` does not need a name because `b` has one
f(b = 2cm, 1, 3cm);
```
