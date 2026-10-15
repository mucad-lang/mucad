# Parameters & Arguments

## Parameters

 > [!IMPORTANT]
 > Parameters in µcad are **not positional** (which means their order is irrelevant)!

Parameters in µcad always have a name which we often call *identifier* and a *type*.

[![test](.test/parameter.svg)](.test/parameter.log)

```mu,parameter
// function with two parameters (`one` and `another`)
fn f(one: Integer, another: Length) {
    print("{one} {another}");
}

// call function with two parameters in arbitrary order
f(another = 2m, one = 1);
```

## Arguments

Arguments are the values which are given in [call](./).
Each argument consists these elements:

- an optional *identifier* in `lower_snake_case`
- a *value* (e.g. `42`, `3.1415`, `"Hello"`)
- a *type* (e.g. `Integer`, `Scalar`, `Length`)
- an optional *unit* that suits the type (e.g. `mm` for `Length`, `m²` for `Area`)

Parameter can have defaults.
Then the notation changes and the type is deduced from the default value.

[![test](.test/parameter_default.svg)](.test/parameter_default.log)

```mu,parameter_default
// function with two parameters (`one`, `another`, and `one_more`)
fn f(one: Integer, another = 2m, one_more: Area) {
    print("{one} {another} {one_more}");
}

// call function with two arguments.
// One matches by name another by type and one_more by default.
f(one = 1, 5m²);
```

There are [several ways](../argument_match/) in which parameters can match [arguments](arguments.md).
