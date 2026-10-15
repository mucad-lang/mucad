# Function Results

Of course, functions can also return a value.
To do so, you need to define the *return type* (using `->`) as shown in the following example:

[![test](.test/function_result.svg)](.test/function_result.log)

```mu,function_result
fn square(x: Scalar) -> Scalar {
    x * x
}

std::assert_eq(square(8.0), 64.0);
```

You may also use `if` to decide between different results.

[![test](.test/function_conditional_result.svg)](.test/function_conditional_result.log)

```mu,function_conditional_result
fn pow(x: Scalar, n: Integer) -> Scalar {
    if n > 1 {
        x * pow(x, n - 1)
    } else if n < 1 {
        1.0 / x / pow(x, -n)
    } else {
        1.0
    }
}

std::assert_eq(pow(8.0, 2), 64.0);
std::assert_eq(pow(8.0, -2), 0.015625);
```

Of course returning a value twice is not allowed.

[![test](.test/return_twice.svg)](.test/return_twice.log)

```mu,return_twice
fn pow(x: Scalar, n: Integer) -> Scalar {
    if n > 1 {
        x * pow(x, n - 1)
    } else if n < 1 {
        1.0 / x / pow(x, -n)
    }
    1.0
}

std::assert_eq(pow(8.0, 2), 64.0);
std::assert_eq(pow(8.0, -2), 0.015625);
```

## Early Return

It is also possible to implement an *early return* pattern with the `return`
statement.

[![test](.test/function_return.svg)](.test/function_return.log)

```mu,function_return
fn pow(x: Scalar, n: Integer) -> Scalar {
    if n > 1 {
        return x * pow(x, n - 1);
    }
    if n < 1 {
        return 1.0 / x / pow(x, -n);
    }
    1.0
}

std::assert_eq(pow(8.0, 2), 64.0);
std::assert_eq(pow(8.0, -2), 0.015625);
```
