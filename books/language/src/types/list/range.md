# Range Expressions

Alternatively a range expression can be used to generate an array with
consecutive values.

> [!IMPORTANT]
> µcad ranges include start **and end point**!
> This is different in many other programming languages.
> So in µcad `[1..3]` results in `[1,2,3]`.

[![test](.test/range_expressions.svg)](.test/range_expressions.log)

```mu,range_expressions
assert_eq([1..5], [1, 2, 3, 4, 5]);
assert_eq([-2..2], [-2, -1, 0, 1, 2]);
```

The order of the endpoints of a range is important.

[![test](.test/range_expressions_bad_order.svg)](.test/range_expressions_bad_order.log)

```mu,range_expressions_bad_order?result=fail
[6..1]; // error
[2..-2]; // error
```

Only `Integer` can be used as endpoint.

[![test](.test/range_expressions_bad_type.svg)](.test/range_expressions_bad_type.log)

```mu,range_expressions_bad_type?result=fail
[1.0..2.0]; // parse_error
```
