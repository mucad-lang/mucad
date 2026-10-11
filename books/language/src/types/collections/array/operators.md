# Array Operators

Arrays support the following operators.

| Operator | Description                         | Example             |
| -------- | ----------------------------------- | ------------------- |
| `+`      | add value to every element          | `[1, 2] + 2`        |
| `-`      | subtract value from every element   | `[1, 2] - 2`        |
| `*`      | multiply every element with a value | `[-1.0, 2.0] * 2.0` |
| `/`      | divide every element by a value     | `[1.0, 2.0] / 2.0`  |
| `-`      | negate every element                | `-[ 1, 2 ]`         |
| `!`      | invert every element                | `![ true, false ]`  |

[![test](.test/array_operations.svg)](.test/array_operations.log)

```mu,array_operations
use std::debug::assert_eq;

assert_eq([[1, 2] + 2, [3, 4]]);
assert_eq([[1, 2] - 2, [-1, 0]]);
assert_eq([[-1.0, 2.0] * 2.0, [-2.0, 4.0]]);
assert_eq([[1.0, 2.0] / 2.0, [0.5, 1.0]]);
assert_eq([-[-1.0, 1.0], [1.0, -1.0]]);
```

[![test](.test/array_operator_not.svg)](.test/array_operator_not.log)

```mu,array_operator_not
std::debug::assert_eq([![true, false], [false, true]]);
```
