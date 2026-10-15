# Operators

> [!NOTE]
> Do not confuse *operators* with [*operations*](../structure/workbenches/types/operations.md)

## Binary operators

[![test](.test/binary_op_examples.svg)](.test/binary_op_examples.log)

```mu,binary_op_examples

std::assert_eq(5 + 6, 11); // Addition
std::assert_eq(5 - 6, -1); // Subtraction
std::assert_eq(5 * 6, 30); // Multiplication
std::assert_eq(5 / 6, 0.83333333333333333); // Division
std::assert_eq(true & false, false); // Logical AND
std::assert_eq(true | false, true); // Logical OR
std::assert_eq(5 > 6, false); // Greater than
std::assert_eq(5 >= 6, false); // Greater or equal
std::assert_eq(5 < 6, true); // Less than
std::assert_eq(5 <= 6, true); // Less or equal
std::assert_eq(5 == 6, false); // Equal
std::assert_eq(5 != 6, true); // Not equal
```

## Unary operators

[![test](.test/unary_op_examples.svg)](.test/unary_op_examples.log)

```mu,unary_op_examples
std::assert_eq(-5, 0 - 5); // Negation
std::assert_eq(!true, false); // Logical NOT
```
