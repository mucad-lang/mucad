# Operators

> [!NOTE]
> Do not confuse *operators* with [*operations*](../structure/workbenches/types/operations.md)

## Binary operators

[![test](.test/binary_op_examples.svg)](.test/binary_op_examples.log)

```mu,binary_op_examples

assert_eq(5 + 6, 11); // Addition
assert_eq(5 - 6, -1); // Subtraction
assert_eq(5 * 6, 30); // Multiplication
assert_eq(5 / 6, 0.83333333333333333); // Division
assert_eq(true & false, false); // Logical AND
assert_eq(true | false, true); // Logical OR
assert_eq(5 > 6, false); // Greater than
assert_eq(5 >= 6, false); // Greater or equal
assert_eq(5 < 6, true); // Less than
assert_eq(5 <= 6, true); // Less or equal
assert_eq(5 == 6, false); // Equal
assert_eq(5 != 6, true); // Not equal
```

## Unary operators

[![test](.test/unary_op_examples.svg)](.test/unary_op_examples.log)

```mu,unary_op_examples
assert_eq(-5, 0 - 5); // Negation
assert_eq(!true, false); // Logical NOT
```
