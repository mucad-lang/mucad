# Quantity Operators

Quantity types support most operators which primitive types do.
See section [operators in expressions](../../expressions/operators.md) for a
complete list

The only difference about quantity operators is, that they calculate units too,
where units remain flexible.

[![test](.test/types_quantity_operators.svg)](.test/types_quantity_operators.log)

```mu,types_quantity_operators

std::assert_eq(6cm * 2cm, 12cm²);
std::assert_eq(6cm / 2cm, 3.0);
std::assert_eq(6cm + 2cm, 80mm);
std::assert_eq(6cm - 2cm, 0.04m);
```
