# Scope tests

## Scopes

[![test](.test/scopes.svg)](.test/scopes.log)

```mu,scopes
a = 1;
use __builtin::debug::assert_valid;
use __builtin::debug::assert_invalid;

assert_valid("a");
assert_invalid("b");
assert_invalid("c");

{
    assert_valid("a");
    assert_invalid("b");
    assert_invalid("c");

    b = 2;

    assert_valid("a");
    assert_valid("b");
    assert_invalid("c");

    c = 3;

    assert_valid("a");
    assert_valid("b");
    assert_valid("c");
};

assert_valid("a");
assert_invalid("b");
assert_invalid("c");
```
