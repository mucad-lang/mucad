# Match Errors

## Missing Arguments

If you do not provide all parameters, you will get an error:

[![test](.test/match_errors.svg)](.test/match_errors.log)

```mu,match_errors#fail
fn f(x: Length, y: Length, z: Length) {}
f(x = 1cm, z = 3cm); // error: y is missing here
```

## Too Many Arguments

When you provide all parameters but some are redundant, you will get a error too:

[![test](.test/match_warnings.svg)](.test/match_warnings.log)

```mu,match_warnings#fail
fn f(x: Length, y: Length, z: Length) {}
f(x = 1cm, y = 2cm, v = 5cm, z = 3cm); // error: Unexpected argument v
```

## Ambiguous Arguments

If some arguments cannot be matched unambiguously to any of the parameters you
will get an error.

[![test](.test/match_ambiguous.svg)](.test/match_ambiguous.log)

```mu,match_ambiguous#fail
fn f(x: Length, y: Length, z: Length) {}
f(x = 1cm, 5cm, 3cm); // error: Missing arguments y and z
```
