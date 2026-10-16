# Functions

[![test](.test/function.svg)](.test/function.log)

```mu,function
// function definition
fn f() {}
// call
f();
```

[![test](.test/function_value.svg)](.test/function_value.log)

```mu,function_value
// parameter
fn f(n: Scalar) {
    // result value
    n+1
}
assert_eq(f(1), 2);
assert_eq(f(4), 5);
```

[![test](.test/function_return_value.svg)](.test/function_return_value.log)

```mu,function_return_value
fn f(n: Scalar) {
    return n+1;
}
assert_eq(f(1), 2);
assert_eq(f(4), 5);
```

[![test](.test/function_if_result.svg)](.test/function_if_result.log)

```mu,function_if_result
fn f(n: Scalar) {
    if n > 3 {
        n-1
    } else {
        n+1
    }
}
assert_eq(f(1), 2);
assert_eq(f(4), 3);
```

[![test](.test/function_mixed.svg)](.test/function_mixed.log)

```mu,function_mixed
fn f(n: Scalar) {
    if n > 3 {
        return n-1;
    }
    n+1
}
assert_eq(f(1), 2);
assert_eq(f(4), 3);
```

[![test](.test/function_missing.svg)](.test/function_missing.log)

```mu,function_missing?mode=todo
fn f(n: Scalar) { // error: not all paths return a value
    if n > 3 {
        n - 1
    }
}

assert_eq(f(1), 2);
assert_eq(f(4), 3);
```
