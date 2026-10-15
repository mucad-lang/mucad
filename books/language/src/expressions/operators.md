# Operators

> [!NOTE]
> Do not confuse *operators* with [*operations*](../structure/workbenches/types/operations.md)

There are several operators which can be used to combine *expressions* with
each other:

| Operator |  Type  | 1st Operand[^x]  | 2nd Operand  | Result Type | Description      |
| :------: | :----: | :--------------- | :----------- | :---------- | ---------------- |
|   `!`    | unary  | `B`              | -            | *same*      | Inversion        |
|   `-`    | unary  | `I` `Q` `A` `T`  | -            | *same*      | Negation         |
|   `+`    | binary | `I` `Q` `A` `T`  | *compatible* | *same*      | Addition         |
|   `-`    | binary | `I` `Q` `A` `T`  | *compatible* | *same*      | Subtraction      |
|   `*`    | binary | `I` `Q` `A` `T`  | *compatible* | *same*      | Multiplication   |
|   `/`    | binary | `I` `Q` `A` `T`  | *compatible* | *same*      | Division         |
|   `^`    | binary | `I` `Q`          | `Integer`    | *like 1st*  | Power            |
|   `&`    | binary | `B`              | `Boolean`    | `Boolean`   | Logical AND      |
|   `\|`   | binary | `B`              | `Boolean`    | `Boolean`   | Logical OR       |
|   `>`    | binary | `I` `Q`          | *compatible* | `Boolean`   | Greater than     |
|   `>=`   | binary | `I` `Q`          | *compatible* | `Boolean`   | Greater or equal |
|   `<`    | binary | `I` `Q`          | *compatible* | `Boolean`   | Less than        |
|   `<=`   | binary | `I` `Q`          | *compatible* | `Boolean`   | Less or equal    |
|   `==`   | binary | `I` `Q` `A` `T`  | *compatible* | `Boolean`   | Equal            |
|   `!=`   | binary | `I` `Q` `A`  `T` | *compatible* | `Boolean`   | Not equal        |

[^x]:`B` = Boolean, `I` = Integer, `Q` = Quantity, `A` = Array, `T` = Tuple

Here are some examples of each operator:

[![test](.test/operator_examples.svg)](.test/operator_examples.log)

```mu,operator_examples

std::assert_eq(-5, 0 - 5); // Negation
std::assert_eq(5 + 6, 11); // Addition
std::assert_eq(5 - 6, -1); // Subtraction
std::assert_eq(5 * 6, 30); // Multiplication
std::assert_eq(5 / 6, 0.83333333333333333); // Division
std::assert_eq(5 ^ 6, 15625); // Power
std::assert_eq(true & false, false); // Logical AND
std::assert_eq(true | false, true); // Logical OR
std::assert_eq(5 > 6, false); // Greater than
std::assert_eq(5 >= 6, false); // Greater or equal
std::assert_eq(5 < 6, true); // Less than
std::assert_eq(5 <= 6, true); // Less or equal
std::assert_eq(5 == 6, false); // Equal
std::assert_eq(5 != 6, true); // Not equal
```

## Operators & Arrays

Some of the operators listed above can be used with arrays too.
There result then is a new array with each value processed with the operator
and the second operand.

See [Array Operators](../types/collections/array/operators.md) for more information.
