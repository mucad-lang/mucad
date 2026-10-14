# Expressions

An *expression* defines a value simply by a [literal](literals.md) or by
combining multiple other expressions.
For example, we can multiply a quantity of 4 millimeters with a factor 5 and
assign it to a constant `v` with the following code:

[![test](.test/expression_multiply.svg)](.test/expression_multiply.log)

```mu,expression_multiply
v = 5 * 4.0mm;

std::assert(v == 20mm);
```

The result of this expression would be `20mm` like the test (see `std::assert`)
proves.
In this form, the computed value is not yet used for anything, so the examples
above will not produce any visible output or effect.

## Expression result types

In the above example `v` is of type `Length` because the expression `5 * 4.0mm`
is the multiplication of a factor (without unit) and a length (in `mm`).

### Scalar & Quantity results

In general units of quantities are calculated according to the operator and
the units of the operands in an expression like the following examples show.

[![test](.test/expression_quantity.svg)](.test/expression_quantity.log)

```mu,expression_quantity

// an integer multiplied with another one remains a integer
std::assert(5 * 4 == 20);

// a scalar multiplied with another scalar or integer results in a scalar
std::assert(5.5 * 4.5 == 24.75);
std::assert(5.1 * 4 == 20.4);
std::assert(5 * 4.1 == 20.5);

// a scalar multiplied with a length is a length
std::assert(5 * 4mm == 20mm);

// two length multiplied with each another is an area
std::assert(5mm * 4mm == 0.2cm²);

// dividing an area by a length is a length
std::assert(20mm² / 4mm == 5mm);
```

### Boolean results

Boolean operations (e.g. `!`, `>`, `==` or `&&`) lead to a boolean result.

[![test](.test/expression_boolean.svg)](.test/expression_boolean.log)

```mu,expression_boolean
// Using logical operators lead to a boolean result
std::assert((5mm > 4mm) == true);
```

Even when using them with models:

[![test](.test/expression_model.svg)](.test/expression_model.log)

```mu,expression_model?mode=todo
// Using logical operators between models too
std::assert((std::geo2d::Circle(1cm) == std::geo2d::Circle(10mm)) == true);
```

Only *Boolean expressions* (expressions with a boolean result) can be used to define conditions (see [if statement](../flow/conditions.md)).
